use std::str::FromStr;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt};
use geo_types::Point;
use itertools::Itertools;
use log::error;
use serde::de;
use tokio::sync::mpsc::Sender;
use tokio::time;
use BusBoardsServer::config::BBConfig;
use crate::api::util::map_feed_entities;
use crate::db::{DBPool, get_stagecoach_trip, get_line_segments};
use crate::GTFSResponder::{STAGECOACH};
use crate::GTFSResponse;
use crate::transit_realtime::{FeedEntity, Position, TripDescriptor, VehiclePosition};
use crate::transit_realtime::trip_descriptor::ScheduleRelationship;
use crate::util::{adjust_timestamp, f64_cmp, get_geo_linepoint_distance, gtfs_date, gtfs_time};

pub async fn stagecoach_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    loop {
        // Get entities for each Stagecoach operator
        let entities = stream::iter(config.stagecoach.regional_operators.iter())
            .then(|(c, gtfs)| get_region(c.as_str(), gtfs.as_str(), &db)).collect::<Vec<Vec<FeedEntity>>>().await.concat();

        // Send to main feed
        tx.send((STAGECOACH, map_feed_entities(&entities), vec![])).await.unwrap_or_else(|err| error!("Could not publish to main feed: {}", err));
        // Wait for next loop
        time::sleep(time::Duration::from_secs(60)).await
    }
}

/// Map journeys for the given Stagecoach region
pub async fn get_region(region: &str, gtfs: &str, db: &Arc<DBPool>) -> Vec<FeedEntity> {
    match reqwest::get(format!("https://api.stagecoach-technology.net/vehicle-tracking/v1/vehicles?services=:{region}:::")).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<StagecoachVehicles>().await {
                    Ok(vehicles) => {
                        return vehicles.services.iter()
                            .filter(|sc| !sc.journey_completed.unwrap_or(false) || sc.cancelled)
                            .filter_map(|sc| {
                                get_stagecoach_trip(
                                    db, gtfs,
                                    sc.line_number.as_str(), sc.next_stop_code.as_str(), &adjust_timestamp(&sc.origin_std?)
                                ).map(|trip| {
                                    let loc = Point::new(sc.longitude, sc.latitude);
                                    let points = get_line_segments(db, trip.route_id.to_string());
                                    let route = &trip.trip_route;
                                    let seqs = &trip.trip_seqs;
                                    let segments: Vec<geo_types::Line<f64>> = (0..route.len()-1).map(|i| {
                                        geo_types::Line::new(points.get(&route[i]).copied().unwrap_or_default(), points.get(&route[i+1]).copied().unwrap_or_default())
                                    }).collect();
                                    let closest_segment = segments.iter().map(|s| get_geo_linepoint_distance(s, &loc))
                                        .position_min_by(f64_cmp).unwrap_or(0);
                                    
                                    FeedEntity {
                                        id: sc.trip_id.to_string(),
                                        is_deleted: None,
                                        trip_update: None,
                                        vehicle: Some(VehiclePosition {
                                            trip: Some(TripDescriptor {
                                                trip_id: Some(trip.trip_id),
                                                route_id: Some(trip.route_id),
                                                direction_id: None,
                                                start_time: Some(gtfs_time(&sc.origin_std.unwrap())),
                                                start_date: Some(gtfs_date(&sc.origin_std.unwrap())),
                                                schedule_relationship: Some(i32::from(if sc.cancelled { ScheduleRelationship::Canceled } else { ScheduleRelationship::Scheduled })),
                                            }),
                                            vehicle: None,
                                            position: Some(Position {
                                                latitude: sc.latitude as f32,
                                                longitude: sc.longitude as f32,
                                                bearing: Some(sc.heading as f32),
                                                odometer: None,
                                                speed: None,
                                            }),
                                            current_stop_sequence: Some((seqs[closest_segment] + 1).min(*seqs.last().unwrap())),
                                            stop_id: Some(route[closest_segment].to_string()),
                                            current_status: None,
                                            timestamp: Some(sc.update_time.timestamp() as u64),
                                            congestion_level: None,
                                            occupancy_status: None,
                                            occupancy_percentage: None,
                                            multi_carriage_details: vec![],
                                        }),
                                        alert: None,
                                        shape: None,
                                    }
                                })
                            }).collect_vec()
                    }
                    Err(err) => {
                        error!("Could not decode Stagecoach data for {region}. {err}");
                    }
                }
            }
        }
        Err(err) => {
            error!("Could not fetch Stagecoach data for {region}. {err}");
        }
    }
    vec![]
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StagecoachVehicles {
    header: Header,
    services: Vec<Service>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    request_id: String,
    returned_item_count: String,
    #[serde(rename = "subscription_id")]
    subscription_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Service {
    #[serde(rename = "fn")]
    fleet_number: String,
    #[serde(rename = "ut", deserialize_with = "deserialize_timestamp_str")]
    update_time: DateTime<Utc>,
    #[serde(rename = "oc")]
    regional_operator: String,
    #[serde(rename = "sn")]
    line_number: String,
    #[serde(rename = "dn")]
    direction: String,
    #[serde(rename = "sd")]
    service_id: String,
    #[serde(rename = "so")]
    local_operator: String,
    #[serde(rename = "sr")]
    service_name: String,
    #[serde(rename = "cd", deserialize_with = "deserialize_bool")]
    cancelled: bool,
    #[serde(rename = "la", deserialize_with = "deserialize_f64")]
    latitude: f64,
    #[serde(rename = "lo", deserialize_with = "deserialize_f64")]
    longitude: f64,
    #[serde(rename = "hg", deserialize_with = "deserialize_usize")]
    heading: usize,
    #[serde(rename = "cg", deserialize_with = "deserialize_usize_opt")]
    calculated_heading: Option<usize>,
    #[serde(rename = "dd")]
    destination: String,
    #[serde(rename = "or")]
    origin_code: String,
    #[serde(rename = "on")]
    origin_name: String,
    #[serde(rename = "nr")]
    next_stop_code: String,
    #[serde(rename = "nn")]
    next_stop_name: String,
    #[serde(rename = "fr")]
    final_stop_code: String,
    #[serde(rename = "fs")]
    final_stop_name: String,
    #[serde(rename = "ao", deserialize_with = "deserialize_timestamp_str_opt")]
    origin_std: Option<DateTime<Utc>>,
    #[serde(rename = "eo", deserialize_with = "deserialize_timestamp_str_opt")]
    origin_etd: Option<DateTime<Utc>>,
    #[serde(rename = "an", deserialize_with = "deserialize_timestamp_str_opt")]
    next_stop_sta: Option<DateTime<Utc>>,
    #[serde(rename = "en", deserialize_with = "deserialize_timestamp_str_opt")]
    next_stop_eta: Option<DateTime<Utc>>,
    #[serde(rename = "ax", deserialize_with = "deserialize_timestamp_str_opt")]
    next_stop_std: Option<DateTime<Utc>>,
    #[serde(rename = "ex", deserialize_with = "deserialize_timestamp_str_opt")]
    next_stop_etd: Option<DateTime<Utc>>,
    #[serde(rename = "af", deserialize_with = "deserialize_timestamp_str_opt")]
    final_stop_sta: Option<DateTime<Utc>>,
    #[serde(rename = "ef", deserialize_with = "deserialize_timestamp_str_opt")]
    final_stop_eta: Option<DateTime<Utc>>,
    #[serde(rename = "ku")]
    kml_url: String,
    #[serde(rename = "td")]
    trip_id: String,
    #[serde(rename = "pr")]
    previous_stop_code: String,
    #[serde(rename = "cs")]
    current_stop: String,
    #[serde(rename = "ns")]
    next_stop: String,
    #[serde(default, rename = "jc", deserialize_with = "deserialize_bool_opt")]
    journey_completed: Option<bool>,
    #[serde(rename = "do", deserialize_with = "deserialize_usize_opt")]
    distance: Option<usize>,
    #[serde(rename = "sg", deserialize_with = "deserialize_f64_opt")]
    stop_latitude: Option<f64>,
    #[serde(rename = "sa", deserialize_with = "deserialize_f64_opt")]
    stop_longitude: Option<f64>,
    #[serde(rename = "to", deserialize_with = "deserialize_usize_opt")]
    total_distance: Option<usize>,
    #[serde(rename = "rg")]
    rag: String,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    match s {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Err(de::Error::unknown_variant(s, &["True", "False"])),
    }
}

fn deserialize_bool_opt<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    match s {
        "" => Ok(None),
        "True" => Ok(Some(true)),
        "False" => Ok(Some(false)),
        _ => Err(de::Error::unknown_variant(s, &["True", "False"])),
    }
}

fn deserialize_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    usize::from_str(s).map_err(|_e| de::Error::custom(format!("String '{}' should be numeric for usize", s)))
}

fn deserialize_usize_opt<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if s.is_empty() { return Ok(None) }
    usize::from_str(s).map(Some).map_err(|_e| de::Error::custom(format!("String '{}' should be numeric for usize", s)))
}

fn deserialize_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    f64::from_str(s).map_err(|_e| de::Error::custom(format!("String '{}' should be numeric for f64", s)))
}

fn deserialize_f64_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if s.is_empty() { return Ok(None) }
    f64::from_str(s).map(Some).map_err(|_e| de::Error::custom(format!("String '{}' should be numeric for f64", s)))
}

fn deserialize_timestamp_str<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let time: usize = deserialize_usize(deserializer)?;
    DateTime::from_timestamp_millis(time as i64).ok_or(de::Error::custom("Time not in range"))
}

fn deserialize_timestamp_str_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if s.is_empty() { return Ok(None); }
    let time = usize::from_str(s).map_err(|_e| de::Error::custom(format!("String '{}' should be numeric for usize", s)))?;
    DateTime::from_timestamp_millis(time as i64).map(Some).ok_or(de::Error::custom("Time not in range"))
}