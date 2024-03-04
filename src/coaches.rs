use chrono::serde::{ts_seconds, ts_seconds_option};
use std::ops::{Add, Sub};
use std::sync::Arc;
use chrono::{DateTime, TimeDelta, Utc};
use futures::{stream, StreamExt};
use geo::EuclideanDistance;
use geo_types::{Line, Point};
use itertools::Itertools;
use tokio::sync::mpsc::Sender;
use tokio::time;
use regex::Regex;
use serde::{Deserialize};
use crate::config::BBConfig;
use crate::db::{CoachRoute, DBPool, get_coach_routes, get_coach_trip, get_line_segments};
use crate::GTFSResponder::{COACHES};
use crate::GTFSResponse;
use crate::transit_realtime::{FeedEntity, Position, TripDescriptor, VehiclePosition};
use crate::transit_realtime::trip_descriptor::ScheduleRelationship::{Canceled, Scheduled};
use crate::transit_realtime::vehicle_position::VehicleStopStatus::InTransitTo;
use crate::util::{f64_cmp, get_url, gtfs_date, gtfs_time};

pub async fn coaches_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    let api_option = get_api_info().await;
    if api_option.is_none() {
        eprintln!("Could not either download or parse coach API data.");
        return;
    }
    let (api_url, api_key) = api_option.unwrap();
    let routes = get_coach_routes(&db, &config);
    loop {
        let time_from = Utc::now().sub(TimeDelta::days(1)).timestamp();
        let time_to = Utc::now().add(TimeDelta::hours(1)).timestamp();

        let r_map = |route: &CoachRoute| get_routes(&db, route.clone(), api_url.as_str(), api_key.as_str(), time_from, time_to);
        let routes: Vec<FeedEntity> = stream::iter(routes.iter())
            .map(r_map)
            .buffer_unordered(10)
            .flat_map(stream::iter)
            .collect::<Vec<FeedEntity>>().await;

        tx.send((COACHES, routes, vec![])).await.unwrap_or_else(|err| eprintln!("{}", err));

        time::sleep(time::Duration::from_secs(60)).await
    }
}

async fn get_routes(db: &Arc<DBPool>, route: CoachRoute, api_url: &str, api_key: &str, time_from: i64, time_to: i64) -> Vec<FeedEntity> {
    match get_url::<MegabusVehicles, _, _>(format!("{api_url}/public-origin-departures-by-route-v1/{}/{time_from}/{time_to}?api_key={api_key}", route.route_short_name), reqwest::Response::json).await {
        Ok(vehicles) => {
            let points = get_line_segments(db, route.route_id.to_string());
            return vehicles.routes[0].chronological_departures.iter()
                .filter_map(|dep| {
                    return if dep.trip.id.ends_with('S') || dep.trip.id.ends_with('E')
                        || dep.active_vehicle.is_none() || dep.tracking.is_completed {
                        None
                    } else if let Some(trip) = get_coach_trip(db, route.route_id.as_str(), &dep.trip.departure_location_name, &dep.trip.arrival_location_name, &dep.trip.departure_time_unix, &dep.trip.arrival_time_unix)
                        && let Some(vehicle) = &dep.active_vehicle {
                        let index = (0..trip.route.len() - 2).map(|i| {
                            Line::new(points[&trip.route[i]], points[&trip.route[i+1]]).euclidean_distance(&Point::<f64>::new(vehicle.current_wgs84_longitude_degrees, vehicle.current_wgs84_latitude_degrees))
                        }).position_min_by(f64_cmp).map(|pos| pos + 1).unwrap_or_default();

                        Some(FeedEntity {
                            id: dep.trip.id.to_string(),
                            is_deleted: None,
                            trip_update: None,
                            vehicle: Some(VehiclePosition {
                                trip: Some(TripDescriptor {
                                    trip_id: Some(trip.trip_id),
                                    route_id: Some(route.route_id.to_string()),
                                    direction_id: None,
                                    start_time: Some(gtfs_time(&dep.trip.departure_time_unix)),
                                    start_date: Some(gtfs_date(&dep.trip.departure_time_unix)),
                                    schedule_relationship: Some(i32::from(if dep.tracking.is_cancelled { Canceled } else { Scheduled })),
                                }),
                                vehicle: None,
                                position: Some(Position {
                                    latitude: vehicle.current_wgs84_latitude_degrees as f32,
                                    longitude: vehicle.current_wgs84_latitude_degrees as f32,
                                    bearing: Some(vehicle.current_forward_azimuth_degrees as f32),
                                    odometer: None,
                                    speed: None,
                                }),
                                current_stop_sequence: Some(trip.seqs[index] as u32),
                                stop_id: Some(trip.route[index].to_string()),
                                current_status: Some(i32::from(InTransitTo)),
                                timestamp: Some(vehicle.last_update_time_unix.timestamp() as u64),
                                congestion_level: None,
                                occupancy_status: None,
                                occupancy_percentage: None,
                                multi_carriage_details: vec![],
                            }),
                            alert: None,
                            shape: None,
                        })
                    } else {
                        None
                    }
                })
                .collect_vec()
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
    vec![]
}

async fn get_api_info() -> Option<(String, String)> {
    let pattern_api_url: Regex = Regex::new(r#"\s*API_URL: '(.*)',"#).unwrap();
    let pattern_api_key: Regex = Regex::new(r#"\s*API_KEY: '(.*)',"#).unwrap();

    if let Ok(config_resp) = reqwest::get("https://coachtracker.uk.megabus.com/configs/global.js").await
        && config_resp.status().is_success()
        && let Ok(config) = config_resp.text().await
        && let Some(captures_url) = pattern_api_url.captures(config.as_str()) && captures_url.len() > 0
        && let Some(captures_key) = pattern_api_key.captures(config.as_str()) && captures_key.len() > 0 {
        Some((captures_url.get(1).unwrap().as_str().to_string(), captures_key.get(1).unwrap().as_str().to_string()))
    } else {
        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct MegabusVehicles {
    code:    usize,
    message: String,
    routes:  Vec<MegabusRoute>,
}

#[derive(Serialize, Deserialize)]
pub struct MegabusRoute {
    metadata:                 Metadata,
    chronological_departures: Vec<ChronologicalDeparture>,
}

#[derive(Serialize, Deserialize)]
pub struct ChronologicalDeparture {
    trip:           MegabusTrip,
    active_vehicle: Option<ActiveVehicle>,
    stop:           MegabusStop,
    tracking:       Tracking,
    coachtracker:   Coachtracker,
}

#[derive(Serialize, Deserialize)]
pub struct ActiveVehicle {
    current_wgs84_latitude_degrees:   f64,
    current_wgs84_longitude_degrees:  f64,
    current_forward_azimuth_degrees:  usize,
    current_speed_mph:                Option<usize>,
    #[serde(with = "ts_seconds")]
    last_update_time_unix:            DateTime<Utc>,
    engine_is_currently_on:           bool,
    engine_is_currently_idling:       bool,
    last_update_time_formatted_local: String,
}

#[derive(Serialize, Deserialize)]
pub struct Coachtracker {
    is_earlier_departure: bool,
    is_later_departure:   bool,
}

#[derive(Serialize, Deserialize)]
pub struct MegabusStop {
    sequence:                                 usize,
    original_source_sequence:                 usize,
    #[serde(with = "ts_seconds")]
    scheduled_arrival_time_unix:              DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    scheduled_departure_time_unix:            DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    live_arrival_time_unix:                   Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    live_departure_time_unix:                 Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    estimated_arrival_time_unix:              Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    estimated_departure_time_unix:            Option<DateTime<Utc>>,
    scheduled_arrival_time_formatted_local:   String,
    scheduled_departure_time_formatted_local: String,
    live_arrival_time_formatted_local:        Option<String>,
    live_departure_time_formatted_local:      Option<String>,
    estimated_arrival_time_formatted_local:   Option<String>,
    estimated_departure_time_formatted_local: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Tracking {
    current_delay_seconds:       Option<isize>,
    total_distance_km:           f64,
    is_future_trip:              bool,
    is_cancelled:                bool,
    is_completed:                bool,
    has_no_tracking:             bool,
    has_no_vehicle:              bool,
    has_no_gps:                  bool,
    is_stationary:               bool,
    is_arrived:                  bool,
    is_arrived_at_current_stop:  bool,
    is_moving:                   bool,
    is_moving_to_current_stop:   bool,
    has_departed_current_stop:   bool,
    has_moved_past_current_stop: bool,
    has_bypassed_current_stop:   bool,
}

#[derive(Serialize, Deserialize)]
pub struct MegabusTrip {
    id:                             String,
    operator_code:                  String,
    operator_name:                  String,
    class_code:                     String,
    class_name:                     String,
    route_id:                       String,
    short_name:                     String,
    direction:                      String,
    pattern_code:                   String,
    duplicate_service:              bool,
    #[serde(with = "ts_seconds")]
    departure_time_unix:            DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    arrival_time_unix:              DateTime<Utc>,
    departure_location_name:        String,
    arrival_location_name:          String,
    departure_locale:               String,
    arrival_locale:                 String,
    duration_seconds:               usize,
    departure_time_formatted_local: String,
    arrival_time_formatted_local:   String,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    route_id:                String,
    short_name:              String,
    departure_location_name: String,
    arrival_location_name:   String,
}