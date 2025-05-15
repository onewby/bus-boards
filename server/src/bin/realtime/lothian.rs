use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Timelike, Utc};
use chrono::serde::ts_seconds;
use futures::{stream, StreamExt};
use futures::future::join_all;
use geo_types::Point;
use itertools::Itertools;
use log::{debug, error, info};
use nu_ansi_term::Color::Yellow;
use reqwest::Client;
use rusqlite::Error;
use tokio::sync::mpsc::Sender;
use tokio::time;

use BusBoardsServer::config::BBConfig;
use crate::api::util::map_feed_entities;
use crate::bus_prediction::{assign_vehicles, get_trip_candidates, get_trip_info, TripCandidate, TripCandidateList, TripInfo};
use crate::db::{DBPool, get_line_segments, get_lothian_patterns_tuples, get_lothian_route, get_lothian_timetabled_trips, get_operator_routes, lothian_trip_query, LothianDBPattern, reset_lothian, save_lothian_pattern_allocations};
use crate::GTFSResponder::LOTHIAN;
use crate::GTFSResponse;
use crate::siri::create_translated_string;
use crate::transit_realtime::{Alert, EntitySelector, FeedEntity, Position, TimeRange, TripDescriptor, VehiclePosition};
use crate::transit_realtime::vehicle_position::VehicleStopStatus;
use crate::util::{adjust_timestamp, get_url, get_url_with_retries, gtfs_date, load_last_update, relative_to, save_last_update, URLParseError};

const UPDATE_FILE: &str = ".update.lothian";

pub async fn lothian_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    let http = Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
    let mut update_time = load_last_update(UPDATE_FILE);

    // Get route-directions to fetch
    let mut all_patterns = get_lothian_patterns_tuples(&db);

    loop {
        // Perform route data updates on first run or at 3am at the configured interval
        if update_time.add(TimeDelta::days(config.update_interval_days as i64)) < Utc::now() {
            info!("{}", Yellow.paint("Performing Lothian route updates"));
            update_route_data(&db, &config).await;
            all_patterns = get_lothian_patterns_tuples(&db);
            info!("{}", Yellow.paint("Lothian route updates completed"));
            let new_update_time = Utc::now().with_hour(3).unwrap().with_minute(0).unwrap();
            update_time = new_update_time;
            save_last_update(UPDATE_FILE, &new_update_time);
        }

        // Match vehicles for each stored pattern
        let p_map = |p: &LothianDBPattern| process_pattern(p.route.to_string(), p.pattern.to_string(), &http, &db);
        let entities = stream::iter(all_patterns.iter())
            .map(p_map)
            .buffer_unordered(10)
            .flat_map(stream::iter)
            .collect::<Vec<FeedEntity>>().await;

        // Publish to main feed
        tx.send((LOTHIAN, map_feed_entities(&entities), vec![])).await.unwrap_or_else(|err| error!("Could not publish to main feed: {}", err));

        // Wait for next fetch
        time::sleep(Duration::from_secs(60)).await
    }
}

/// Convert located trip to GTFS
fn to_feed_entity(trip: &TripInfo, vehicle: &LothianVehicle, candidate: &TripCandidate) -> FeedEntity {
    FeedEntity {
        id: format!("lothian-{}-{}", vehicle.service_name, vehicle.journey_id),
        is_deleted: None,
        trip_update: None,
        vehicle: Some(VehiclePosition {
            trip: Some(TripDescriptor {
                trip_id: Some(candidate.trip_id.to_string()),
                route_id: None,
                direction_id: None,
                start_time: Some(candidate.times[0].format("%H:%M:%S").to_string()),
                start_date: Some(candidate.date.to_string()),
                schedule_relationship: None,
            }),
            vehicle: None,
            position: Some(Position {
                latitude: vehicle.latitude as f32,
                longitude: vehicle.longitude as f32,
                bearing: Some(vehicle.heading as f32),
                odometer: None,
                speed: None,
            }),
            current_stop_sequence: Some(candidate.seqs[trip.stop_index]),
            stop_id: Some(candidate.route[trip.stop_index].to_owned()),
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

/// Match vehicles for a specific route direction
async fn process_pattern(route: String, pattern: String, http: &Client, db: &Arc<DBPool>) -> Vec<FeedEntity> {
    return match http.get(format!("https://tfeapp.com/api/website/vehicles_on_route.php?route_id={pattern}")).send().await {
        Ok(resp) => {
            return if resp.status().is_success() && let Ok(vehicles) = resp.json::<LothianLiveVehicles>().await {
                let now_date = adjust_timestamp(&Utc::now());
                // Get list of possible trips stored in GTFS that could match with a realtime vehicle
                let candidates = get_trip_candidates(db, pattern.as_str(), &now_date, lothian_trip_query);
                // Get route stance locations for vehicles to be matched to their nearest route line segment
                let points = get_line_segments(db, route.to_string());
                // For each vehicle, get a list of how delayed a vehicle would be on each trip at its current location
                let mut closeness: Vec<TripCandidateList> = vehicles.vehicles.iter().enumerate().map(|(v_i, v)| {
                    TripCandidateList {
                        vehicle: v_i,
                        cands: candidates.iter().enumerate().map(|(c_i, c)| get_trip_info(c, c_i, &points, &Point::new(v.longitude, v.latitude), &now_date)).collect(),
                    }
                }).filter(|v| !v.cands.is_empty()).collect();

                // Match vehicles trip-by-trip using the most on-time matches first
                let v: Vec<_> = assign_vehicles(&mut closeness, &candidates).iter().map(|(&i, trip)| to_feed_entity(trip, &vehicles.vehicles[i], &candidates[trip.candidate])).collect();
                v
            } else {
                vec![]
            }
        }
        Err(err) => {
            error!("Lothian processing error for pattern {pattern}: {}", err);
            vec![]
        }
    };
}

/// Map Lothian disruptions data to GTFS
pub async fn get_lothian_disruptions(db: &Arc<DBPool>) -> Vec<Alert> {
    return if let Ok(resp) = reqwest::get("https://lothianupdates.com/api/public/getServiceUpdates").await
        && resp.status().is_success() && let Ok(disruptions) = resp.json::<LothianEvents>().await {
        disruptions.events.iter().map(|event| {
            Alert {
                active_period: event.time_ranges.iter().map(|time_range| {
                    TimeRange {
                        start: DateTime::<Utc>::from_str(time_range.start.as_str()).map(|t| t.timestamp() as u64).ok(),
                        end: time_range.finish.as_ref().and_then(|str| DateTime::<Utc>::from_str(str).ok()).map(|t| t.timestamp() as u64)
                    }
                }).collect(),
                informed_entity: event.routes_affected.iter().map(|route| {
                    EntitySelector {
                        agency_id: None,
                        route_id: get_lothian_route(db, route.name.to_string()),
                        route_type: None,
                        trip: None,
                        stop_id: None,
                        direction_id: None,
                    }
                }).collect(),
                url: Some(create_translated_string(event.url.to_string())),
                header_text: Some(create_translated_string(event.title.en.to_string())),
                description_text: Some(create_translated_string(event.description.en.to_string())),
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
        }).collect()
    } else {
        vec![]
    }
}

/// Match Lothian journey codes to GTFS trip IDs
pub async fn update_route_data(db: &Arc<DBPool>, config: &Arc<BBConfig>) {
    reset_lothian(db);
    match get_url::<LothianRoutes, _, _>("https://lothianapi.com/routes", reqwest::Response::json).await {
        Ok(routes) => {
            join_all(
                routes.groups.iter().map(|group| process_group(db, config, group))
            ).await;
        }
        Err(e) => { error!("Could not get Lothian routes: {}", e) }
    };
}

/// Match Lothian journey codes to GTFS trip IDs for a given Lothian operator
pub async fn process_group(db: &Arc<DBPool>, config: &Arc<BBConfig>, group: &LothianGroup) {
    let gtfs = get_operator_routes(db, config.lothian.operators.get(group.id.as_str()).or(config.lothian.operators.get("lothian")).unwrap().as_str());
    join_all(group.routes.iter().filter_map(|r| {
        gtfs.iter()
            .find(|(route_id, route_name)| r.name == *route_name)
            .map(|(route_id, route_name)| (r, route_id))
    }).map(|r| process_route(db, r))).await;
}

/// Match Lothian journey codes to GTFS trip IDs for a given route
pub async fn process_route(db: &Arc<DBPool>, (route, gtfs_route_id): (&LothianRoute, &String)) {
    match get_url_with_retries::<LothianPatterns, _, _>(format!("https://lothianapi.com/routePatterns?route_name={}", route.name), reqwest::Response::json, 2).await {
        Ok(patterns) => {
            for p in patterns.patterns {
                if let Err(e) = process_route_pattern(&db, gtfs_route_id, p.id.as_str()).await {
                    error!("Error processing pattern {} ({gtfs_route_id}): {}", p.id, e)
                }
            }
        }
        Err(e) => { error!("Could not process Lothian route {}: {}", route.name, e); }
    };
}

/// Match Lothian journey codes to GTFS trip IDs for a given route pattern
pub async fn process_route_pattern(db: &Arc<DBPool>, gtfs_route_id: &str, pattern: &str) -> Result<(), Error> {
    let current_date = Utc::now();
    let allocateds = stream::iter(0..7)
        .map(|i| current_date.add(TimeDelta::days(i)))
        .then(|date| get_url_with_date(pattern, date))
        .filter_map(|r| async move {
            if r.is_err() {
                error!("Error processing Lothian route pattern {gtfs_route_id}, {pattern} - {}", r.as_ref().unwrap_err());
            }
            r.map(|(date, timetables)| {
                let gtfs_trips = get_lothian_timetabled_trips(db, &date, gtfs_route_id);
                let trip_ids = timetables.timetable.trips.iter().filter_map(|trip| {
                    let deps = trip.departures.iter().filter(|dep| dep.time != "-" && dep.scheduled_for.is_some()).collect_vec();
                    let origin = deps.first().unwrap();
                    let dest = deps.last().unwrap();
                    let origin_date = origin.scheduled_for.as_ref().unwrap().unix_time;
                    let origin_time = adjust_timestamp(&relative_to(&origin_date, &origin_date));
                    let dest_time = adjust_timestamp(&relative_to(&origin_date, &dest.scheduled_for.as_ref().unwrap().unix_time));
                    // Find trip with matching origin/dest locations + times
                    gtfs_trips.iter().find(|trip|
                        trip.origin_stop == origin.stop_id && trip.dest_stop == dest.stop_id
                            && trip.min_stop_time == origin_time.timestamp() && trip.max_stop_time == dest_time.timestamp())
                        .map(|t| t.trip_id.to_string())
                }).collect_vec();

                stream::iter(trip_ids)
            }).ok()
        }).flat_map(|r| r).collect::<Vec<_>>().await;
    debug!("{gtfs_route_id}, {pattern} - saving {}", allocateds.len());
    save_lothian_pattern_allocations(db, pattern, &allocateds, gtfs_route_id).inspect_err(|e| error!("Could not save Lothian allocations for {pattern}: {e}"))
}

/// Utility function - perform timetable fetch for route pattern, return with the date searched for
async fn get_url_with_date(pattern: &str, date: DateTime<Utc>) -> Result<(DateTime<Utc>, LothianTimetables), URLParseError> {
    get_url_with_retries::<LothianTimetables, _, _>(format!("https://lothianapi.com/timetable?route_pattern_id={}&date={}", pattern, gtfs_date(&date)), reqwest::Response::json, 2)
        .await.map(|url| (date, url))
}

#[derive(Serialize, Deserialize, Debug)]
struct LothianLiveVehicles {
    vehicles: Vec<LothianVehicle>
}

#[derive(Serialize, Deserialize, Debug)]
struct LothianVehicle {
    vehicle_id:   String,
    vehicle_type: String,
    journey_id:   String,
    latitude:     f64,
    longitude:    f64,
    destination:  String,
    heading:      u64,
    service_name: String,
    next_stop_id: String
}

#[derive(Serialize, Deserialize)]
struct LothianEvents {
    events: Vec<LothianEvent>
}

#[derive(Serialize, Deserialize)]
struct LothianEvent {
    id:              String,
    created:         String,
    last_updated:    Option<String>,
    cause:           String,
    effect:          String,
    severity:        String,
    title:           LothianDescription,
    description:     LothianDescription,
    time_ranges:     Vec<LothianTimeRange>,
    url:             String,
    webarticle_html: String,
    routes_affected: Vec<LothianRoutesAffected>
}

#[derive(Serialize, Deserialize)]
struct LothianDescription {
    en: String
}

#[derive(Serialize, Deserialize)]
struct LothianRoutesAffected {
    name: String
}

#[derive(Serialize, Deserialize)]
struct LothianTimeRange {
    start: String,
    finish: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct LothianRoutes {
    server:      String,
    #[serde(rename = "timeElapsed")]
    time_elapsed: f64,
    #[serde(rename = "networkTime")]
    network_time: String,
    groups:      Vec<LothianGroup>
}

#[derive(Serialize, Deserialize)]
pub struct LothianGroup {
    id:          String,
    name:        String,
    description: Option<String>,
    routes:      Vec<LothianRoute>
}

#[derive(Serialize, Deserialize)]
pub struct LothianRoute {
    id:          String,
    name:        String,
    description: String,
    color:       String,
    #[serde(rename = "textColor")]
    text_color:   String
}

#[derive(Serialize, Deserialize)]
pub struct LothianPatterns {
    server:      String,
    #[serde(rename = "timeElapsed")]
    time_elapsed: f64,
    #[serde(rename = "networkTime")]
    network_time: String,
    route:       LothianRoute,
    patterns:    Vec<LothianPattern>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LothianPattern {
    id:          String,
    #[serde(rename = "routeName")]
    route_name:   String,
    origin:      String,
    destination: String,
    polyline:    Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LothianTimetables {
    server:      String,
    #[serde(rename = "timeElapsed")]
    time_elapsed: f64,
    #[serde(rename = "networkTime")]
    network_time: String,
    timetable:   LothianTimetable
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LothianTimetable {
    #[serde(rename = "routePattern")]
    route_pattern: LothianPattern,
    trips:        Vec<LothianTrip>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LothianTrip {
    #[serde(rename = "tripID")]
    trip_id:     String,
    departures: Vec<LothianDeparture>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LothianDeparture {
    #[serde(rename = "stopID")]
    stop_id:        String,
    time:          String,
    #[serde(rename = "isTimingPoint")]
    is_timing_point: bool,
    #[serde(rename = "scheduledFor")]
    scheduled_for:  Option<ScheduledFor>,
    sequence:      Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScheduledFor {
    #[serde(rename = "unixTime", with = "ts_seconds")]
    unix_time:    DateTime<Utc>,
    #[serde(rename = "isoTime")]
    iso_time:     String,
    #[serde(rename = "displayTime")]
    display_time: String
}