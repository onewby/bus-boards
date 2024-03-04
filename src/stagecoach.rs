use std::str::FromStr;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt};
use itertools::Itertools;
use serde::de;
use tokio::sync::mpsc::Sender;
use tokio::time;
use crate::config::BBConfig;
use crate::db::{DBPool, get_stagecoach_trip};
use crate::GTFSResponder::{STAGECOACH};
use crate::GTFSResponse;
use crate::transit_realtime::{FeedEntity, Position, TripDescriptor, VehiclePosition};
use crate::transit_realtime::trip_descriptor::ScheduleRelationship;
use crate::util::{gtfs_date, gtfs_time};

pub async fn stagecoach_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    loop {
        let entities = stream::iter(config.stagecoach.regional_operators.iter())
            .then(|c| get_region(c.as_str(), &config, &db)).collect::<Vec<Vec<FeedEntity>>>().await.concat();

        tx.send((STAGECOACH, entities, vec![])).await.unwrap_or_else(|err| eprintln!("{}", err));
        time::sleep(time::Duration::from_secs(60)).await
    }
}

pub async fn get_region(region: &str, config: &Arc<BBConfig>, db: &Arc<DBPool>) -> Vec<FeedEntity> {
    match reqwest::get(format!("https://api.stagecoach-technology.net/vehicle-tracking/v1/vehicles?services=:{region}:::")).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<StagecoachVehicles>().await {
                    Ok(vehicles) => {
                        return vehicles.services.iter()
                            .filter(|sc| !(sc.journey_completed && !sc.cancelled))
                            .filter_map(|sc| {
                                get_stagecoach_trip(
                                    &db, config.stagecoach.local_operators[&sc.local_operator.to_lowercase()].as_str(),
                                    sc.line_number.as_str(), sc.next_stop_code.as_str(), &sc.origin_std?
                                ).map(|trip| {
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
                                            current_stop_sequence: Some(trip.stop_seq as u32),
                                            stop_id: Some(sc.next_stop_code.to_string()),
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
                        eprintln!("Could not decode Stagecoach data for {region}. {err}");
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Could not fetch Stagecoach data for {region}. {err}");
        }
    }
    return vec![]
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
    #[serde(rename = "jc", deserialize_with = "deserialize_bool")]
    journey_completed: bool,
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

fn deserialize_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    return usize::from_str(s).map_err(|e| de::Error::custom(format!("String '{}' should be numeric for usize", s)))
}

fn deserialize_usize_opt<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if s == "" { return Ok(None) }
    return usize::from_str(s).map(|s| Some(s)).map_err(|e| de::Error::custom(format!("String '{}' should be numeric for usize", s)))
}

fn deserialize_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    return f64::from_str(s).map_err(|e| de::Error::custom(format!("String '{}' should be numeric for f64", s)))
}

fn deserialize_f64_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if s == "" { return Ok(None) }
    return f64::from_str(s).map(|s| Some(s)).map_err(|e| de::Error::custom(format!("String '{}' should be numeric for f64", s)))
}

fn deserialize_timestamp_str<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let time: usize = deserialize_usize(deserializer)?;
    return DateTime::from_timestamp_millis(time as i64).ok_or(de::Error::custom("Time not in range"))
}

fn deserialize_timestamp_str_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    if s == "" { return Ok(None); }
    let time = usize::from_str(s).map_err(|e| de::Error::custom(format!("String '{}' should be numeric for usize", s)))?;
    return DateTime::from_timestamp_millis(time as i64).map(|d| Some(d)).ok_or(de::Error::custom("Time not in range"))
}