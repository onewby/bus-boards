use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use futures::{FutureExt, stream, StreamExt};
use geo_types::Point;
use itertools::Itertools;
use reqwest::{Client};
use tokio::sync::mpsc::Sender;
use tokio::time;
use crate::config::BBConfig;
use crate::db::{DBPool, get_line_segments, get_lothian_patterns_tuples, get_lothian_route, lothian_trip_query, LothianPattern};
use crate::GTFSResponder::LOTHIAN;
use crate::GTFSResponse;
use crate::bus_prediction::{assign_vehicles, get_trip_candidates, get_trip_info, TripCandidate, TripCandidateList, TripInfo};
use crate::siri::create_translated_string;
use crate::transit_realtime::{Alert, EntitySelector, FeedEntity, Position, TimeRange, TripDescriptor, VehiclePosition};
use crate::transit_realtime::vehicle_position::VehicleStopStatus;

pub async fn lothian_listener(tx: Sender<GTFSResponse>, _: Arc<BBConfig>, db: Arc<DBPool>) {
    let http = Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
    let all_patterns = get_lothian_patterns_tuples(&db);
    loop {
        let p_map = |p: &LothianPattern| process_pattern(p.route.to_string(), p.pattern.to_string(), &http, &db);
        let entities = stream::iter(all_patterns.iter())
            .map(p_map)
            .buffer_unordered(10)
            .flat_map(|p| stream::iter(p))
            .collect::<Vec<FeedEntity>>().await;

        tx.send((LOTHIAN, entities, vec![])).await.unwrap_or_else(|err| eprintln!("{}", err));

        time::sleep(Duration::from_secs(60)).await
    }
}

fn to_feed_entity(trip: &TripInfo, vehicle: &LothianVehicle, candidates: &Vec<TripCandidate>) -> FeedEntity {
    FeedEntity {
        id: format!("lothian-{}-{}", vehicle.service_name, vehicle.journey_id.to_string()),
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
                latitude: vehicle.latitude as f32,
                longitude: vehicle.longitude as f32,
                bearing: Some(vehicle.heading as f32),
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

async fn process_pattern(route: String, pattern: String, http: &Client, db: &Arc<DBPool>) -> Vec<FeedEntity> {
    return match http.get(format!("https://tfeapp.com/api/website/vehicles_on_route.php?route_id={pattern}")).send().await {
        Ok(resp) => {
            return if resp.status().is_success() && let Ok(vehicles) = resp.json::<LothianLiveVehicles>().await {
                let now_date = Utc::now();
                let candidates = get_trip_candidates(&db, pattern.as_str(), &now_date, lothian_trip_query);
                let points = get_line_segments(&db, route.to_string());
                let mut closeness: Vec<TripCandidateList> = vehicles.vehicles.iter().enumerate().map(|(v_i, v)| {
                    TripCandidateList {
                        vehicle: v_i,
                        cands: candidates.iter().enumerate().map(|(c_i, c)| get_trip_info(c, c_i, &points, &Point::new(v.longitude, v.latitude), &now_date)).collect(),
                    }
                }).filter(|v| v.cands.len() > 0).collect();
                assign_vehicles(&mut closeness, &candidates).iter().map(|(&i, trip)| to_feed_entity(trip, &vehicles.vehicles[i], &candidates)).collect()
            } else {
                vec![]
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            vec![]
        }
    };
}

pub async fn get_lothian_disruptions(db: &Arc<DBPool>) -> Vec<Alert> {
    return if let Ok(resp) = reqwest::get("https://lothianupdates.com/api/public/getServiceUpdates").await
        && resp.status().is_success() && let Ok(disruptions) = resp.json::<LothianEvents>().await {
        disruptions.events.iter().map(|event| {
            Alert {
                active_period: event.time_ranges.iter().map(|time_range| {
                    TimeRange {
                        start: DateTime::<Utc>::from_str(time_range.start.as_str()).map(|t| t.timestamp() as u64).ok(),
                        end: time_range.finish.as_ref().map(|str| DateTime::<Utc>::from_str(str).ok()).flatten().map(|t| t.timestamp() as u64)
                    }
                }).collect(),
                informed_entity: event.routes_affected.iter().map(|route| {
                    EntitySelector {
                        agency_id: None,
                        route_id: get_lothian_route(&db, route.name.to_string()),
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