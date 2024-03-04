use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use chrono::{DateTime, DurationRound, Utc};
use config::Map;
use futures::{FutureExt, StreamExt};
use geo_types::Point;
use itertools::{Itertools};
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex};
use tokio::{time};
use tokio_stream::{self as stream};
use crate::config::{BBConfig, OperatorName, PassengerSource, SourceURL};
use crate::db::{DBPool, get_line_segments, get_route_id, passenger_trip_query};
use crate::GTFSResponder::PASSENGER;
use crate::{bus_prediction, GTFSResponse};
use crate::bus_prediction::{TripCandidate, TripCandidateList, TripInfo};
use crate::siri::create_translated_string;
use crate::transit_realtime::{Alert, EntitySelector, FeedEntity, Position, TimeRange, TripDescriptor, VehiclePosition};
use crate::transit_realtime::vehicle_position::VehicleStopStatus;

pub async fn passenger_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    loop {
        let entities_stream = stream::iter(config.passenger.iter()).then(|s| get_source_vehicles(s, &db));
        let entities = entities_stream.collect::<Vec<Vec<FeedEntity>>>().await.concat();

        tx.send((PASSENGER, entities, vec![])).await.unwrap_or_else(|err| eprintln!("{}", err));

        time::sleep(time::Duration::from_secs(60)).await
    }
}

pub async fn get_passenger_disruptions(db: &Arc<DBPool>, config: &Arc<BBConfig>, alerts_cache: &Mutex<HashMap<SourceURL, Vec<Alert>>>) -> Vec<Alert> {
    let alerts_stream = stream::iter(config.passenger.iter()).then(|s| get_source_alerts(s, alerts_cache, db));
    alerts_stream.collect::<Vec<Vec<Alert>>>().await.concat()
}

pub async fn get_source_vehicles((url, operators): (&SourceURL, &Map<OperatorName, PassengerSource>), db: &Arc<DBPool>) -> Vec<FeedEntity> {
    if let Ok(vehicles_resp) = reqwest::get(format!("{url}/network/vehicles")).await {
        let vehicles_resp_str = vehicles_resp.text().await.unwrap();
        let vehicles_result: serde_json::Result<PassengerVehicles> = serde_json::from_str(vehicles_resp_str.as_str());
        if vehicles_result.is_ok() {
            let vehicle_features = vehicles_result.unwrap().features;
            return vehicle_features.into_iter().group_by(|v| (v.properties.operator.to_string(), v.properties.line.to_string())).into_iter()
                .filter(|((operator, line), _)| operators.contains_key(&operator.to_lowercase()))
                .flat_map(|source| process_line_vehicles(db, operators, source.0, source.1))
                .collect()
        } else {
            eprintln!("{}", vehicles_result.err().unwrap());
        }
    }
    vec![]
}

pub fn process_line_vehicles<'a, 'b, FeatureIterator>(db: &Arc<DBPool>, operators: &Map<OperatorName, PassengerSource>, (operator, line): (String, String), vehicles_iter: FeatureIterator) -> Vec<FeedEntity>
    where FeatureIterator: Iterator<Item = VehiclesFeature> {
    let operator_data = operators.get(&operator.to_lowercase()).unwrap();
    let route_id_result = get_route_id(db, operator_data.gtfs.to_owned(), line.to_owned());
    if route_id_result.is_err() { return vec![] }
    let route_id = route_id_result.unwrap();
    let now_date = Utc::now();

    let vehicles: Vec<VehiclesFeature> = vehicles_iter.collect();
    let candidates = bus_prediction::get_trip_candidates(db, route_id.as_str(), &now_date, passenger_trip_query);
    let points = get_line_segments(db, route_id);
    let mut closeness: Vec<TripCandidateList> = vehicles.iter().enumerate().map(|(i, vehicle)| gather_direction_candidates(&now_date, &candidates, &points, i, &vehicle)).filter(|v| v.cands.len() > 0).collect();
    bus_prediction::assign_vehicles(&mut closeness, &candidates).iter().map(|(&i, trip)| to_feed_entity(trip, &vehicles[i], &candidates)).collect()
}

fn gather_direction_candidates<'a>(now_date: &DateTime<Utc>, candidates: &'a Vec<TripCandidate>, points: &'a Map<String, Point>, i: usize, vehicle: &'a VehiclesFeature) -> TripCandidateList {
    let direction: u8 = if vehicle.properties.direction == "inbound" { 0 } else { 1 };
    TripCandidateList {
        vehicle: i,
        cands: candidates.iter().enumerate().filter(|(i, c)| c.direction == Some(direction)).map(|(i, c)| bus_prediction::get_trip_info(c, i, &points, &vehicle.geometry.coordinates, &now_date)).collect(),
    }
}

fn to_feed_entity(trip: &TripInfo, vehicle: &VehiclesFeature, candidates: &Vec<TripCandidate>) -> FeedEntity {
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

pub async fn get_source_alerts((url, operators): (&SourceURL, &Map<OperatorName, PassengerSource>), alerts_cache: &Mutex<HashMap<SourceURL, Vec<Alert>>>, db: &Arc<DBPool>) -> Vec<Alert> {
    if let Ok(disruptions_resp) = reqwest::get(format!("{url}/network/disruptions")).await && disruptions_resp.status().is_success() {
        let disruptions: PassengerDisruptions = disruptions_resp.json().await.unwrap();
        let alerts = disruptions.embedded.alert.iter().map(|alert| {
            Alert {
                active_period: alert.active_periods.iter().map(|active_period| {
                    TimeRange {
                        start: DateTime::<Utc>::from_str(active_period.start.as_str()).map(|t| t.timestamp() as u64).ok(),
                        end: active_period.end.as_ref().map(|str| DateTime::<Utc>::from_str(str).ok()).flatten().map(|t| t.timestamp() as u64)
                    }
                }).collect(),
                informed_entity: alert.embedded.line.iter().filter_map(|line| {
                    let operator = &line.embedded.transmodel_operator.name;
                    let agency_id = &operators.get(operator.as_str())?.gtfs;
                    let route = &line.name;
                    let route_id = get_route_id(&db, agency_id.to_string(), route.to_string()).ok()?;
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
                url: alert.links.as_ref().map(|link| create_translated_string(link.info.href.to_string())),
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
    links: Option<AlertLinks>
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
    assets: Assets,
    shapes: Assets,
    stops: Assets,
    changes: Changes,
    timetable: Timetable,
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