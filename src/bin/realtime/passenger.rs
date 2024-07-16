use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, DurationRound, TimeDelta, Timelike, Utc};
use config::Map;
use futures::{stream, StreamExt};
use futures::future::join_all;
use geo_types::Point;
use itertools::Itertools;
use log::{debug, error, info};
use nu_ansi_term::Color::Yellow;
use reqwest::StatusCode;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time;

use bus_prediction::{assign_vehicles, get_trip_candidates};
use BusBoardsServer::config::{BBConfig, OperatorName, PassengerSource, SourceURL};

use crate::{bus_prediction, GTFSResponse};
use crate::bus_prediction::{TripCandidate, TripCandidateList, TripInfo};
use crate::db::{DBPool, get_line_segments, get_operator_routes, get_passenger_route_trips, get_route_id, passenger_trip_query, PassengerRouteTrip, reset_passenger, RouteID, RouteName, save_passenger_trip_allocations};
use crate::GTFSResponder::PASSENGER;
use crate::siri::create_translated_string;
use crate::transit_realtime::{Alert, EntitySelector, FeedEntity, Position, TimeRange, TripDescriptor, VehiclePosition};
use crate::transit_realtime::vehicle_position::VehicleStopStatus;
use crate::util::{adjust_timestamp, get_url, load_last_update, relative_to, save_last_update, URLParseError};

const UPDATE_FILE: &str = ".update.passenger";
const DIRECTIONS: [&str; 2] = ["inbound", "outbound"];

pub async fn passenger_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    let mut update_time = load_last_update(UPDATE_FILE);
    loop {
        // Perform route updates on first run or at 2am on each interval
        if update_time.add(TimeDelta::days(config.update_interval_days as i64)) < Utc::now() {
            info!("{}", Yellow.paint("Performing Passenger route updates"));
            update_passenger_data(&db, &config).await;
            info!("{}", Yellow.paint("Passenger route updates completed"));
            let new_update_time = Utc::now().with_hour(2).unwrap().with_minute(0).unwrap();
            update_time = new_update_time;
            save_last_update(UPDATE_FILE, &new_update_time);
        }

        // Get data for each operator
        let entities_stream = stream::iter(config.passenger.iter()).then(|s| get_source_vehicles(s, &db));
        let entities = entities_stream.collect::<Vec<Vec<FeedEntity>>>().await.concat();

        // Publish to main feed
        tx.send((PASSENGER, entities, vec![])).await.unwrap_or_else(|err| error!("{}", err));

        // Wait for next loop
        time::sleep(time::Duration::from_secs(60)).await
    }
}

/// Map disruptions for each operator to GTFS, flatten into one feed
pub async fn get_passenger_disruptions(db: &Arc<DBPool>, config: &Arc<BBConfig>, alerts_cache: &Mutex<HashMap<SourceURL, Vec<Alert>>>) -> Vec<Alert> {
    let alerts_stream = stream::iter(config.passenger.iter()).then(|s| get_source_alerts(s, alerts_cache, db));
    alerts_stream.collect::<Vec<Vec<Alert>>>().await.concat()
}

/// Get realtime data for a given Passenger operator(s) feed
pub async fn get_source_vehicles((url, operators): (&SourceURL, &Map<OperatorName, PassengerSource>), db: &Arc<DBPool>) -> Vec<FeedEntity> {
    // Fetch feed vehicle data
    if let Ok(vehicles_resp) = reqwest::get(format!("{url}/network/vehicles")).await {
        let vehicles_resp_str = vehicles_resp.text().await.unwrap();
        let vehicles_result: serde_json::Result<PassengerVehicles> = serde_json::from_str(vehicles_resp_str.as_str());
        if vehicles_result.is_ok() {
            let vehicle_features = vehicles_result.unwrap().features;
            // Get results for each operator the feed contains
            return vehicle_features.into_iter().group_by(|v| (v.properties.operator.to_string(), v.properties.line.to_string())).into_iter()
                .filter(|((operator, _line), _)| operators.contains_key(&operator.to_lowercase()))
                .flat_map(|source| process_line_vehicles(db, operators, source.0, source.1))
                .collect()
        } else {
            error!("{}", vehicles_result.err().unwrap());
        }
    }
    vec![]
}

/// Map vehicles on a given route to GTFS
pub fn process_line_vehicles<FeatureIterator>(db: &Arc<DBPool>, operators: &Map<OperatorName, PassengerSource>, (operator, line): (String, String), vehicles_iter: FeatureIterator) -> Vec<FeedEntity>
    where FeatureIterator: Iterator<Item = VehiclesFeature> {
    // Get GTFS route ID for the given route
    let operator_data = operators.get(&operator.to_lowercase()).unwrap();
    let route_id_result = get_route_id(db, operator_data.gtfs.to_owned(), line.to_owned());
    if route_id_result.is_err() { return vec![] }
    let route_id = route_id_result.unwrap();

    let now_date = adjust_timestamp(&Utc::now());
    // Get list of vehicles specific to this operator
    let vehicles: Vec<VehiclesFeature> = vehicles_iter.collect();

    // Get list of possible trips stored in GTFS that could match with a realtime vehicle
    let candidates = get_trip_candidates(db, route_id.as_str(), &now_date, passenger_trip_query);
    // Get route stance locations for vehicles to be matched to their nearest route line segment
    let points = get_line_segments(db, route_id);
    // For each vehicle, get a list of how delayed a vehicle would be on each trip at its current location
    let mut closeness: Vec<TripCandidateList> = vehicles.iter().enumerate()
        .map(|(i, vehicle)| gather_direction_candidates(&now_date, &candidates, &points, i, vehicle))
        .filter(|v| !v.cands.is_empty())
        .collect();
    // Match vehicles trip-by-trip using the most on-time matches first
    assign_vehicles(&mut closeness, &candidates).iter()
        // then map to GTFS feed entities
        .map(|(&i, trip)| to_feed_entity(trip, &vehicles[i], &candidates)).collect()
}

/// Get expected delay and next stop for each vehicle-trip combination in the specified direction
fn gather_direction_candidates<'a>(now_date: &DateTime<Utc>, candidates: &'a [TripCandidate], points: &'a Map<String, Point>, i: usize, vehicle: &'a VehiclesFeature) -> TripCandidateList {
    let direction: u8 = if vehicle.properties.direction == "inbound" { 0 } else { 1 }; // map to the database's encoding for direction
    TripCandidateList {
        vehicle: i,
        cands: candidates.iter().enumerate().filter(|(_i, c)| c.direction == Some(direction))
            .map(|(i, c)| bus_prediction::get_trip_info(c, i, points, &vehicle.geometry.coordinates, now_date)).collect(),
    }
}

/// Map vehicle-trip assignment to GTFS
fn to_feed_entity(trip: &TripInfo, vehicle: &VehiclesFeature, candidates: &[TripCandidate]) -> FeedEntity {
    FeedEntity {
        id: format!("{}-{}-{}", vehicle.properties.operator, vehicle.properties.line, vehicle.properties.vehicle),
        is_deleted: None,
        trip_update: None,
        vehicle: Some(VehiclePosition {
            trip: Some(TripDescriptor {
                trip_id: Some(candidates[trip.candidate].trip_id.to_string()),
                route_id: None,
                direction_id: None,
                start_time: Some(candidates[trip.candidate].times[0].format("%H:%M:%S").to_string()),
                start_date: Some(candidates[trip.candidate].date.to_string()),
                schedule_relationship: None,
            }),
            vehicle: None,
            position: Some(Position {
                latitude: vehicle.geometry.coordinates.y() as f32,
                longitude: vehicle.geometry.coordinates.x() as f32,
                bearing: vehicle.properties.bearing.map(|i| i as f32),
                odometer: None,
                speed: None,
            }),
            current_stop_sequence: Some(candidates[trip.candidate].seqs[trip.stop_index]),
            stop_id: Some(candidates[trip.candidate].route[trip.stop_index].to_owned()),
            current_status: Some(i32::from(VehicleStopStatus::InTransitTo)),
            timestamp: Some(Utc::now().timestamp() as u64),
            congestion_level: None,
            occupancy_status: None,
            occupancy_percentage: None,
            multi_carriage_details: vec![],
        }),
        alert: None,
        shape: None,
    }
}

/// Get disruptions for a given operator feed
pub async fn get_source_alerts((url, operators): (&SourceURL, &Map<OperatorName, PassengerSource>), alerts_cache: &Mutex<HashMap<SourceURL, Vec<Alert>>>, db: &Arc<DBPool>) -> Vec<Alert> {
    if let Ok(disruptions_resp) = reqwest::get(format!("{url}/network/disruptions")).await && disruptions_resp.status().is_success() {
        let disruptions: PassengerDisruptions = disruptions_resp.json().await.expect(format!("{url} disruptions").as_str());
        let alerts = disruptions.embedded.alert.iter().map(|alert| {
            Alert {
                active_period: alert.active_periods.iter().map(|active_period| {
                    TimeRange {
                        start: DateTime::<Utc>::from_str(active_period.start.as_str()).map(|t| t.timestamp() as u64).ok(),
                        end: active_period.end.as_ref().and_then(|str| DateTime::<Utc>::from_str(str).ok()).map(|t| t.timestamp() as u64)
                    }
                }).collect(),
                informed_entity: alert.embedded.line.iter().filter_map(|line| {
                    let operator = &line.embedded.transmodel_operator.name;
                    let agency_id = &operators.get(operator.as_str())?.gtfs;
                    let route = &line.name;
                    let route_id = get_route_id(db, agency_id.to_string(), route.to_string()).ok()?;
                    // get GTFS route ID
                    Some(EntitySelector {
                        agency_id: None,
                        route_id: Some(route_id),
                        route_type: None,
                        trip: None,
                        stop_id: None,
                        direction_id: None,
                    })
                }).collect(),
                url: alert.links.as_ref().map(|link| create_translated_string(link.links_self.href.to_string())),
                header_text: Some(create_translated_string(alert.header.to_string())),
                description_text: Some(create_translated_string(alert.description.to_string())),
                cause: None,
                effect: None,
                tts_header_text: None,
                tts_description_text: None,
                severity_level: None,
                image: None,
                image_alternative_text: None,
                cause_detail: None,
                effect_detail: None,
            }
        }).collect();
        alerts_cache.lock().await.insert(url.to_owned(), alerts);
    }
    alerts_cache.lock().await.get(url).unwrap_or(&vec![]).clone()
}

/// Update Passenger journey to GTFS trip mappings
pub async fn update_passenger_data(db: &Arc<DBPool>, config: &Arc<BBConfig>) {
    reset_passenger(db);
    join_all(config.passenger.iter()
        .map(|source| update_operator(db, source))).await;
}

/// Update journey-trip mappings for a given Passenger feed
pub async fn update_operator(db: &Arc<DBPool>, (source_url, operators): (&SourceURL, &Map<OperatorName, PassengerSource>)) {
    // Get list of lines in this feed
    match get_url::<PassengerLines, _, _>(format!("{source_url}/network/lines"), reqwest::Response::json).await {
        Ok(lines) => {
            for (op_name, op_source) in operators {
                // Get list of all routes in GTFS
                let routes = get_operator_routes(db, op_source.gtfs.as_str());
                // Match GTFS route to Passenger line, then collect journeys for each of the next 7 days
                let results = stream::iter(routes.iter().filter_map(|route| match_line(&lines, op_name, op_source, &route)))
                    .then(|route| get_days_info(db, source_url, op_source, route))
                    .flat_map(|v| stream::iter(v))
                    .collect::<Vec<PassengerDirectionInfo>>().await;
                // Save to database
                save_passenger_trip_allocations(&db, &results).expect(&format!("Could not save allocations for {op_name}."));
            }
        }
        Err(err) => { error!("Could not get line data for {source_url}: {err}.") }
    }
}

/// GTFS route to Passenger line mapping
#[derive(Clone)]
pub struct RouteInfo {
    pub gtfs_id: String,
    pub gtfs_name: String,
    pub online_name: String
}

/// Match GTFS route to Passenger line by name/operator
fn match_line(lines: &PassengerLines, op_name: &str, op_source: &PassengerSource, (route_id, route_name): &(RouteID, RouteName)) -> Option<RouteInfo> {
    if let Some(line) = lines.embedded.line.iter().find(|l| l.name.to_lowercase() == route_name.to_lowercase()
        && l.embedded.transmodel_operator.code == op_source.op_code) {
        debug!("- {} {route_name} ({})", op_source.op_code, line.name);
        Some(RouteInfo {
            gtfs_id: route_id.to_string(),
            gtfs_name: route_name.to_string(),
            online_name: line.name.to_string(),
        })
    } else {
        debug!("No route exists on web data for {op_name}/${route_name}");
        None
    }
}

/// Mapping of GTFS route to Passenger journey and the direction it operates in on its Passenger route
pub struct PassengerDirectionInfo {
    pub gtfs: String,
    pub polar: String,
    pub direction: u8
}

/// Get trip-journey mappings for the next 7 days on a given route
async fn get_days_info(db: &Arc<DBPool>, source_url: &SourceURL, operator: &PassengerSource, route: RouteInfo) -> Vec<PassengerDirectionInfo> {
    let today = Utc::now();
    // for each day, in both directions, get trips
    stream::iter(0..7)
        .then(|i| {
            let route_arc = Arc::new(route.clone());
            let day = today.add(TimeDelta::days(i));
            let gtfs_routes = get_passenger_route_trips(db, &day, route.gtfs_id.as_str());
            let gtfs_routes_arc = Arc::new(gtfs_routes);
            stream::iter(DIRECTIONS)
                .then(move |dir| {
                    let route_arc = route_arc.clone();
                    let gtfs_routes_arc = gtfs_routes_arc.clone();
                    // Get for a specific day (then flatten into one return Vec)
                    async move { get_day_direction_info(source_url, operator, route_arc, day, dir, gtfs_routes_arc).await }
                })
                .flat_map(stream::iter)
                .collect::<Vec<_>>()
        })
        .flat_map(stream::iter)
        .collect::<Vec<_>>().await
}

/// Get trip-journey mappings for a specific day on a given route
async fn get_day_direction_info(source_url: &SourceURL, operator: &PassengerSource, route: Arc<RouteInfo>, day: DateTime<Utc>, direction: &str, gtfs_routes: Arc<Vec<PassengerRouteTrip>>) -> Vec<PassengerDirectionInfo> {
    // Map Passenger direction (inbound/outbound) to database encoding of 0/1
    let (dir_index, _) = DIRECTIONS.iter().find_position(|&&dir| dir == direction).unwrap();
    match get_url::<PassengerTimetable, _, _>(format!("{source_url}/network/operators/{}/lines/{}/timetables?direction={direction}&date={}", operator.op_code, route.online_name, day.format("%Y-%m-%d")), reqwest::Response::json).await {
        Ok(timetable) => {
            timetable.embedded.journey.iter()
                // Get journeys that match the actual line we're looking for
                .filter(|tj| tj.links.line.name == route.online_name)
                .filter_map(|tj| {
                    let origin_date = tj.embedded.visit.first().unwrap().aimed_departure_time.unwrap();
                    let origin_time = adjust_timestamp(&relative_to(&origin_date, &origin_date)).duration_trunc(TimeDelta::seconds(1)).unwrap().timestamp();
                    let dest_time = adjust_timestamp(&relative_to(&origin_date, &tj.embedded.visit.last().unwrap().aimed_arrival_time)).duration_trunc(TimeDelta::seconds(1)).unwrap().timestamp();
                    let dest_time_m1 = dest_time - 60;

                    // Match GTFS trips based on origin/dest times
                    gtfs_routes.iter()
                        .find(|r| r.min_stop_time == origin_time && r.max_stop_time == dest_time)
                        // If we can't find anything, try (dest time - 1 minute) to account for rounding
                        .or_else(|| gtfs_routes.iter()
                            .find(|r| r.min_stop_time == origin_time && r.max_stop_time == dest_time_m1))
                        .map(|trip| {
                            PassengerDirectionInfo {
                                gtfs: trip.trip_id.to_string(),
                                polar: tj.id.to_string(),
                                direction: dir_index as u8,
                            }
                        })
                }).collect_vec()
        }
        Err(err) => {
            if let URLParseError::StatusCodeError(StatusCode::NOT_FOUND) = err {} else {
                error!("Could not get timetable data for {} {} {} on {}: {}", operator.op_code, route.online_name, direction, day, err);
            }
            vec![]
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PassengerVehicles {
    #[serde(rename = "type")]
    passenger_vehicles_type: String,
    features: Vec<VehiclesFeature>,
}

#[derive(Serialize, Deserialize)]
pub struct VehiclesFeature {
    #[serde(rename = "type")]
    feature_type: String,
    geometry: Geometry,
    properties: VehiclesProperties,
    #[serde(rename = "_embedded")]
    embedded: VehiclesEmbedded,
    #[serde(rename = "_links")]
    links: VehiclesLinks,
}

#[derive(Serialize, Deserialize)]
pub struct VehiclesEmbedded {
    #[serde(rename = "transmodel:line")]
    transmodel_line: VehiclesTransmodelLine,
}

#[derive(Serialize, Deserialize)]
pub struct VehiclesTransmodelLine {
    id: String,
    name: String,
    title: String,
    description: String,
    colors: Colors,
    href: String,
}

#[derive(Serialize, Deserialize)]
pub struct Colors {
    background: String,
    foreground: String,
    background_secondary: Option<String>,
    foreground_secondary: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    geometry_type: String,
    coordinates: Point,
}

#[derive(Serialize, Deserialize)]
pub struct VehiclesLinks {
    topups: Topups,
}

#[derive(Serialize, Deserialize)]
pub struct Topups {
    href: String,
    title: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehiclesProperties {
    direction: String,
    line: String,
    operator: String,
    vehicle: String,
    href: String,
    meta: Option<VehiclesMeta>,
    bearing: Option<i64>,
    compass_direction: Option<String>,
    destination: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct VehiclesMeta {
    fleet_number: String,
    low_emission_engine: Option<bool>,
    low_floor: Option<bool>,
    make: String,
    model: String,
    number_plate: String,
    payments_contactless: bool,
    tenant: Option<String>,
    #[serde(rename = "type")]
    meta_type: String,
    wheelchair_capacity: i64,
    name: Option<String>,
    power_usb: Option<bool>,
    wifi: Option<bool>,
    next_stop_announcements: Option<bool>,
    double_glazing: Option<bool>,
    next_stop_display: Option<bool>,
    zero_emission_engine: Option<bool>,
    coat_hooks: Option<bool>,
    luggage_racks: Option<bool>,
    reading_lights: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct PassengerDisruptions {
    #[serde(rename = "_embedded")]
    embedded: PassengerDisruptionsEmbedded,
}

#[derive(Serialize, Deserialize)]
pub struct PassengerDisruptionsEmbedded {
    alert: Vec<PassengerAlert>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerAlert {
    id: String,
    header: String,
    description: String,
    cause: String,
    effect: String,
    created: String,
    #[serde(rename = "type")]
    alert_type: String,
    active_periods: Vec<ActivePeriod>,
    #[serde(rename = "_embedded")]
    embedded: AlertEmbedded,
    #[serde(rename = "_links")]
    links: Option<Links>
}

#[derive(Serialize, Deserialize)]
pub struct ActivePeriod {
    start: String,
    time_range_display: String,
    end: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AlertLinks {
    info: LinksHref
}

#[derive(Serialize, Deserialize)]
pub struct LinksHref {
    href: String
}

#[derive(Serialize, Deserialize)]
pub struct AlertEmbedded {
    #[serde(default = "Vec::new")]
    line: Vec<Line>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    id: String,
    name: String,
    title: String,
    description: String,
    tenant: String,
    detail: Option<String>,
    colors: Colors,
    start_date: String,
    end_date: String,
    #[serde(rename = "_links")]
    links: Links,
    #[serde(rename = "_embedded")]
    embedded: LineEmbedded,
    weighting: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct LineEmbedded {
    #[serde(rename = "transmodel:operator")]
    transmodel_operator: TransmodelOperator,
}

#[derive(Serialize, Deserialize)]
pub struct TransmodelOperator {
    code: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    links_self: Assets,
    assets: Option<Assets>,
    shapes: Option<Assets>,
    stops: Option<Assets>,
    changes: Option<Changes>,
    timetable: Option<Timetable>,
    #[serde(rename = "transmodel:line")]
    transmodel_line: Option<Vec<DisruptionTransmodelLine>>,
}

#[derive(Serialize, Deserialize)]
pub struct Assets {
    href: String,
}

#[derive(Serialize, Deserialize)]
pub struct Changes {
    href: String,
    #[serde(rename = "type")]
    changes_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct Timetable {
    href: String,
    line: String,
    direction: String,
    date: String,
    operator: String,
}

#[derive(Serialize, Deserialize)]
pub struct DisruptionTransmodelLine {
    href: String,
    id: String,
    name: String,
    title: String,
    operator: String,
}

#[derive(Serialize, Deserialize)]
pub struct PassengerLines {
    #[serde(rename = "_embedded")]
    pub embedded: PolarLinesEmbedded
}

#[derive(Serialize, Deserialize)]
pub struct PolarLinesEmbedded {
    #[serde(rename = "transmodel:line")]
    pub line: Vec<LinesTransmodelLine>
}

#[derive(Serialize, Deserialize)]
pub struct LinesTransmodelLine {
    id:          String,
    name:        String,
    description: String,
    colors:      Colors,
    #[serde(rename = "_embedded")]
    embedded:   LineEmbedded
}

#[derive(Serialize, Deserialize)]
pub struct PassengerTimetable {
    #[serde(rename = "_links")]
    links:    PassengerTimetableLinks,
    #[serde(rename = "_embedded")]
    embedded: PassengerTimetableEmbedded,
    date:      String
}

#[derive(Serialize, Deserialize)]
pub struct PassengerTimetableEmbedded {
    #[serde(rename = "transmodel:line")]
    line: Vec<LinesTransmodelLine>,
    #[serde(rename = "transmodel:direction")]
    direction: TransmodelDirection,
    #[serde(rename = "timetable:journey", default = "Vec::new")]
    journey: Vec<PassengerTimetableJourney>,
}

#[derive(Serialize, Deserialize)]
pub struct PassengerTimetableJourney {
    id:        String,
    #[serde(rename = "_embedded")]
    embedded: TimetableJourneyEmbedded,
    #[serde(rename = "_links")]
    links:    TimetableJourneyLinks,
}

#[derive(Serialize, Deserialize)]
pub struct TimetableJourneyEmbedded {
    #[serde(rename = "timetable:visit")]
    visit: Vec<PassengerTimetableVisit>,
}

#[derive(Serialize, Deserialize)]
pub struct PassengerTimetableVisit {
    #[serde(rename = "aimedArrivalTime")]
    aimed_arrival_time: DateTime<Utc>,
    #[serde(rename = "aimedDepartureTime")]
    aimed_departure_time: Option<DateTime<Utc>>,
    #[serde(rename = "_links")]
    links: PassengerTimetableVisitLinks
}

#[derive(Serialize, Deserialize)]
pub struct PassengerTimetableVisitLinks {
    #[serde(rename = "timetable:waypoint")]
    waypoint: PassengerSelf
}

#[derive(Serialize, Deserialize)]
pub struct PassengerSelf {
    href: String
}

#[derive(Serialize, Deserialize)]
pub struct TimetableJourneyLinks {
    #[serde(rename = "transmodel:line")]
    line: LinksTransmodelLine
}

#[derive(Serialize, Deserialize)]
pub struct LinksTransmodelLine {
    name:        String,
    description: String,
    colors:      Colors,
    href:        String
}

#[derive(Serialize, Deserialize)]
pub struct TransmodelDirection {
    name:        String,
    origin:      String,
    destination: String
}

#[derive(Serialize, Deserialize)]
pub struct PassengerTimetableLinks {
    #[serde(rename = "transmodel:line")]
    line: Vec<LinksTransmodelLine>,
    #[serde(rename = "transmodel:direction")]
    direction: Vec<TransmodelDirection>,
    #[serde(rename = "self")]
    passenger_self: PassengerSelf,
    switch: Switch
}

#[derive(Serialize, Deserialize)]
pub struct Switch {
    href:      String,
    templated: bool
}