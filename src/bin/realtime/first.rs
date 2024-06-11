use std::cmp::min;
use std::{fmt, fs};
use std::fmt::{Display, Formatter, Write};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz::Europe__London;
use futures::{SinkExt, StreamExt};
use geo_types::Point;
use itertools::Itertools;
use reqwest::Client;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;

use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::time;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::tungstenite::error::UrlError;
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use url::Url;
use uuid::Uuid;
use BusBoardsServer::config::BBConfig;
use BusBoardsServer::RPCConfiguration;
use crate::db::{DBPool, get_first_trip};
use crate::GTFSResponder::FIRST;
use crate::GTFSResponse;
use crate::transit_realtime::{FeedEntity, Position, TripDescriptor, VehiclePosition};
use crate::transit_realtime::trip_descriptor::ScheduleRelationship::Scheduled;

const REGIONS_FILE: &str = "first-regions.json";

pub async fn first_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    let mut regions: Vec<RPCConfiguration> = if let Ok(regions_file) = File::open(REGIONS_FILE)
        && let Ok(regions) = serde_json::from_reader::<_, Vec<RPCConfigParams>>(BufReader::new(regions_file)) {
        regions.iter().map(|RPCConfigParams(region)| *region).collect_vec()
    } else {
        println!("Could not find existing FirstBus regions file - creating a default.");
        let regions = config.first.bounds.values().cloned().collect_vec();
        save_regions(&regions);
        regions
    };

    loop {
        let mut ws = initialise_ws_until_success(config.first.api_key.as_str()).await;
        println!("FirstBus websocket successfully connected");

        tx.send((FIRST, get_vehicles(&mut ws, &db, &config, &mut regions).await.unwrap_or(vec![]), vec![])).await.unwrap_or_else(|err| eprintln!("{}", err));
        time::sleep(Duration::from_secs(60)).await;

        println!("FirstBus connection lost - attempting reconnect");
    }
}

async fn get_vehicles(ws: &mut WSStream, db: &Arc<DBPool>, config: &Arc<BBConfig>, regions: &mut Vec<RPCConfiguration>) -> Option<Vec<FeedEntity>> {
    let (new_regions, vehicles) = get_vehicles_by_regions(ws, regions).await;
    if regions.len() != new_regions.len() {
        *regions = new_regions;
        save_regions(regions);
    }

    Some(vehicles.iter()
        .filter_map(|v|
            get_first_trip(db, v.line_name.as_str(),
                           config.first.operators[&v.operator.to_lowercase()].as_str(),
                           v.origin_atcocode.as_str(), &v.stops[0].get_time())
                .map(|t| (v, t)))
        .map(map_to_feed_entity).collect())
}

async fn get_vehicles_by_regions(ws: &mut WSStream, regions: &[RPCConfiguration]) -> (Vec<RPCConfiguration>, Vec<Member>) {
    let mut region_queue = regions.to_owned();
    let mut final_regions: Vec<RPCConfiguration> = Vec::with_capacity(region_queue.len());
    let mut final_vehicles: Vec<Member> = vec![];
    while let Some(region) = region_queue.pop() {
        if let Some(resp) = send_and_receive(ws, &region).await && let Some(vehicles) = resp.resource.member {
            if vehicles.len() >= 50 {
                let height = region.max_lat - region.min_lat;
                let width = region.max_lon - region.min_lon;

                let lat_middle = vehicles.iter().map(|v| v.status.location.coordinates.y()).sum::<f64>() / (vehicles.len() as f64);
                let lon_middle = vehicles.iter().map(|v| v.status.location.coordinates.x()).sum::<f64>() / (vehicles.len() as f64);

                if (0.5 - (lat_middle-region.min_lat).abs()/height).abs() < (0.5 - (lon_middle-region.min_lon).abs()/width).abs() {
                    // lat is more central
                    region_queue.push(RPCConfiguration {
                        min_lon: region.min_lon, max_lon: region.max_lon, min_lat: region.min_lat, max_lat: lat_middle
                    });
                    region_queue.push(RPCConfiguration {
                        min_lon: region.min_lon, max_lon: region.max_lon, min_lat: lat_middle, max_lat: region.max_lat
                    });
                } else {
                    region_queue.push(RPCConfiguration {
                        min_lon: region.min_lon, max_lon: lon_middle, min_lat: region.min_lat, max_lat: region.max_lat
                    });
                    region_queue.push(RPCConfiguration {
                        min_lon: lon_middle, max_lon: region.max_lon, min_lat: region.min_lat, max_lat: region.max_lat
                    });
                }
            } else {
                final_regions.push(region);
                final_vehicles.extend(vehicles);
            }
        } else {
            final_regions.push(region);
        }
    }
    (final_regions, final_vehicles)
}

fn map_to_feed_entity((v, trip_id): (&Member, String)) -> FeedEntity {
    FeedEntity {
        id: v.status.vehicle_id.to_string(),
        is_deleted: None,
        trip_update: None,
        vehicle: Some(VehiclePosition {
            trip: Some(TripDescriptor {
                trip_id: Some(trip_id.to_string()),
                route_id: None,
                direction_id: None,
                start_time: Some(format!("{}:00", v.stops[0].time)),
                start_date: Some(v.stops[0].date.to_string()),
                schedule_relationship: Some(i32::from(Scheduled)),
            }),
            vehicle: None,
            position: Some(Position {
                latitude: v.status.location.coordinates.y() as f32,
                longitude: v.status.location.coordinates.x() as f32,
                bearing: Some(v.status.bearing as f32),
                odometer: None,
                speed: None,
            }),
            current_stop_sequence: Some(v.status.stops_index.value as u32),
            stop_id: Some(v.stops[v.status.stops_index.value].atcocode.to_string()),
            current_status: None,
            timestamp: Some(v.status.recorded_at_time.timestamp() as u64),
            congestion_level: None,
            occupancy_status: None,
            occupancy_percentage: None,
            multi_carriage_details: vec![],
        }),
        alert: None,
        shape: None,
    }
}

async fn send_and_receive(ws: &mut WSStream, region: &RPCConfiguration) -> Option<FirstVehicles> {
    let uuid = Uuid::new_v4().to_string();
    ws.flush().await.ok()?;
    let msg = serde_json::to_string(
        &RPCConfigurationRequest {
            jsonrpc: "2.0".to_string(),
            id: uuid.clone(),
            method: "configuration".to_string(),
            params: *region
        }
    ).unwrap();
    ws.send(Message::Text(msg)).await.ok()?;

    let mut current_id: String = "".to_string();
    while let Some(msg_option) = ws.next().await {
        if let Ok(ref msg) = msg_option && let Ok(msg) = msg.to_text() {
            let resp: serde_json::Result<RPCRequest> = serde_json::from_str(msg);
            if let Ok(request) = resp {
                match request {
                    RPCRequest::Result(res) => {
                        current_id = res.id.to_string();
                    }
                    RPCRequest::Update(upd) => {
                        if current_id == uuid {
                            return Some(upd.params);
                        }
                    }
                    RPCRequest::Error(err) => {
                        eprintln!("{:?}", err.error);
                    }
                    _ => {}
                }
            } else {
                eprintln!("{:?}", resp.unwrap_err());
            }
        } else {
            eprintln!("{}", msg_option.unwrap_err());
        }
    }

    None
}

fn get_client_request(url_str: &str, access_token: &str) -> tokio_tungstenite::tungstenite::Result<Request> {
    let url = Url::parse(url_str).unwrap();
    let authority = url.authority();
    let host = authority
        .find('@')
        .map(|idx| authority.split_at(idx + 1).1)
        .unwrap_or_else(|| authority);

    if host.is_empty() {
        return Err(Error::Url(UrlError::EmptyHostName));
    }

    let req = Request::builder()
        .method("GET")
        .header("Host", host)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .uri(url_str)
        .body(())?;
    Ok(req)
}

async fn initialise_ws(api_key: &str) -> Option<WSStream> {
    return if let Ok(resp) = Client::new().get("https://prod.mobileapi.firstbus.co.uk/api/v2/bus/service/socketInfo")
                        .header("x-app-key", api_key).send().await
        && resp.status().is_success()
        && let Ok(token_resp) = resp.json::<FirstWebSocketInfo>().await {
        let request = get_client_request("wss://streaming.bus.first.transportapi.com/", token_resp.data.access_token.as_str()).unwrap();
        let ws_stream_option = connect_async(request).await;
        if ws_stream_option.is_ok() {
            let (ws_stream, _) = ws_stream_option.unwrap();
            Some(ws_stream)
        } else {
            eprintln!("{}", ws_stream_option.unwrap_err());
            None
        }
    } else {
        None
    }
}

const MAX_TIMEOUT: u64 = 32;
async fn initialise_ws_until_success(api_key: &str) -> WSStream {
    let mut timeout = 1;
    loop {
        if let Some(ws_result) = initialise_ws(api_key).await {
            break ws_result
        }
        println!("Could not connect to FirstBus websocket - retrying");
        time::sleep(Duration::from_secs(timeout)).await;
        timeout = min(MAX_TIMEOUT, timeout * 2);
    }
}

fn save_regions(regions: &[RPCConfiguration]) {
    fs::write(REGIONS_FILE, serde_json::to_vec(&regions.iter().map(|r| RPCConfigParams(*r)).collect_vec()).unwrap()).unwrap();
}

struct RPCConfigParams(RPCConfiguration);

struct ConfigVisitor;

impl<'de> Visitor<'de> for ConfigVisitor
{
    /// Return type of this visitor. This visitor computes the max of a
    /// sequence of values of type T, so the type of the maximum is T.
    type Value = RPCConfigParams;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("four floats in sequence")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<RPCConfigParams, S::Error>
        where
            S: SeqAccess<'de>,
    {
        Ok(RPCConfigParams(
            RPCConfiguration {
                min_lat: seq.next_element()?.ok_or(de::Error::custom("No min lat"))?,
                min_lon: seq.next_element()?.ok_or(de::Error::custom("No min lon"))?,
                max_lat: seq.next_element()?.ok_or(de::Error::custom("No max lat"))?,
                max_lon: seq.next_element()?.ok_or(de::Error::custom("No max lon"))?
            }
        ))
    }
}

impl<'de> Deserialize<'de> for RPCConfigParams {
    fn deserialize<D>(deserializer: D) -> Result<RPCConfigParams, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ConfigVisitor)
    }
}

impl Serialize for RPCConfigParams
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.0.min_lat)?;
        seq.serialize_element(&self.0.min_lon)?;
        seq.serialize_element(&self.0.max_lat)?;
        seq.serialize_element(&self.0.max_lon)?;
        seq.end()
    }
}

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RPCRequest {
    Configuration(RPCConfigurationRequest),
    Result(RPCResult),
    Update(RPCUpdate),
    Error(RPCError),
    Skip(Nothing)
}

impl Display for RPCRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RPCRequest::Configuration(_) => { f.write_str("Configuration") }
            RPCRequest::Result(res) => { f.write_fmt(format_args!("Result {}", res.id)) }
            RPCRequest::Update(_) => { f.write_str("Update") }
            RPCRequest::Error(err) => { f.write_fmt(format_args!("Error {}", err.error.code)) }
            RPCRequest::Skip(_) => { f.write_str("Skip") }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCConfigurationRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: RPCConfiguration
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCResult {
    jsonrpc: String,
    id: String,
    result: Nothing
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Nothing {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCUpdate {
    jsonrpc: String,
    method: String,
    params: FirstVehicles
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCError {
    jsonrpc: String,
    id: String,
    error: RPCErrorBody
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCErrorBody {
    code: i64,
    data: String,
    message: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirstVehicles {
    resource: Resource
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    member: Option<Vec<Member>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Member {
    dir:             String,
    line:            String,
    line_name:       String,
    operator:        String,
    operator_name:   String,
    origin_atcocode: String,
    request_time:    DateTime<Utc>,
    status:          Status,
    stops:           Vec<Stop>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    bearing:                isize,
    location:               FirstLocation,
    occupancy:              Occupancy,
    progress_between_stops: ProgressBetweenStops,
    recorded_at_time:       DateTime<Utc>,
    stops_index:            StopsIndex,
    vehicle_id:             String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirstLocation {
    #[serde(deserialize_with = "crate::util::deserialize_point_array")]
    coordinates: Point<f64>,
    #[serde(rename = "type")]
    loc_type:        String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Occupancy {
    types: Vec<Type>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Type {
    capacity: isize,
    name:     String,
    occupied: isize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressBetweenStops {
    value: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StopsIndex {
    #[serde(rename = "type")]
    stop_type: String,
    value: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stop {
    aimed:        Aimed,
    atcocode:     String,
    bearing:      Option<String>,
    date:         String,
    indicator:    String,
    latitude:     f64,
    locality:     String,
    longitude:    f64,
    name:         String,
    smscode:      String,
    stop_name:    String,
    time:         String,
    timing_point: bool
}

impl Stop {
    fn get_time(&self) -> DateTime<Utc> {
        NaiveDateTime::parse_from_str(format!("{} {}", self.date, self.time).as_str(), "%Y-%m-%d %H:%M").unwrap()
            .and_local_timezone(Europe__London).single()
            .map(|tz| tz.with_timezone(&Utc))
            .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Aimed {
    arrival:   Arrival,
    departure: Arrival
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Arrival {
    date: Option<String>,
    time: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct FirstWebSocketInfo {
    data: Data
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    url:            String,
    #[serde(rename = "access-token")]
    access_token: String
}