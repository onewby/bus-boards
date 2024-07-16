#![feature(let_chains)]
#![feature(async_closure)]

mod bods;
mod disruptions;
mod db;
mod siri;
mod ember;
mod passenger;
mod util;
mod bus_prediction;
mod lothian;
mod coaches;
mod stagecoach;
mod first;

#[macro_use]
extern crate serde;

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime};
use axum::extract::State;
use axum::Router;
use axum::routing::{get};
use log::{debug, info};
use nu_ansi_term::Color::{Green, Red};
use prost::Message;
use tokio::sync::{mpsc};
use tokio::sync::mpsc::{Sender};
use tower_http::{compression::CompressionLayer};
use crate::bods::bods_listener;
use crate::coaches::coaches_listener;
use BusBoardsServer::config::{BBConfig, load_config};
use BusBoardsServer::GTFSResponder;
use crate::db::{DBPool, open_db};
use crate::disruptions::{disruptions_listener};
use crate::ember::ember_listener;
use crate::first::first_listener;
use crate::GTFSResponder::{BODS, COACHES, DISRUPTIONS, EMBER, FIRST, LOTHIAN, PASSENGER, STAGECOACH};
use crate::lothian::lothian_listener;
use crate::passenger::passenger_listener;
use crate::stagecoach::stagecoach_listener;
use crate::transit_realtime::{Alert, FeedEntity, FeedMessage};

pub mod transit_realtime {
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

type GTFSResponse = (GTFSResponder, Vec<FeedEntity>, Vec<Alert>);
type GTFSVehicles = HashMap<GTFSResponder, Vec<FeedEntity>>;
type GTFSAlerts = HashMap<GTFSResponder, Vec<Alert>>;

struct GTFSState {
    vehicles: RwLock<GTFSVehicles>,
    alerts: RwLock<GTFSAlerts>
}

impl Default for GTFSState {
    fn default() -> Self {
        GTFSState {
            vehicles: RwLock::new(HashMap::new()),
            alerts: RwLock::new(HashMap::new())
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = load_config();

    // Spawn thread looking for responses from each data retriever
    let gtfs_state = Arc::new(GTFSState::default());
    let (tx, mut rx) = mpsc::channel::<GTFSResponse>(16);
    let gtfs_ref = gtfs_state.clone();
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            debug!("Received from {}", response.0);
            gtfs_ref.vehicles.write().unwrap().insert(response.0, response.1);
            gtfs_ref.alerts.write().unwrap().insert(response.0, response.2);
        }
    });

    // Initialise database
    let db = open_db();
    let arc_cfg = Arc::new(config);
    let arc_db = Arc::new(db);

    // Spawn data retrievers for each provider on a separate thread
    spawn_listener(&arc_cfg, BODS, &tx, bods_listener);
    spawn_listener(&arc_cfg, EMBER, &tx, ember_listener);
    spawn_listener_db(&arc_cfg, PASSENGER, &tx, &arc_db.clone(), passenger_listener);
    spawn_listener_db(&arc_cfg, DISRUPTIONS, &tx, &arc_db.clone(), disruptions_listener);
    spawn_listener_db(&arc_cfg, LOTHIAN, &tx, &arc_db.clone(), lothian_listener);
    spawn_listener_db(&arc_cfg, STAGECOACH, &tx, &arc_db.clone(), stagecoach_listener);
    spawn_listener_db(&arc_cfg, COACHES, &tx, &arc_db.clone(), coaches_listener);
    spawn_listener_db(&arc_cfg, FIRST, &tx, &arc_db.clone(), first_listener);

    // Serve API endpoints
    let app = Router::new()
        .route("/api/gtfsrt/proto", get(gtfs_realtime_proto))
        .route("/api/gtfsrt/json", get(gtfs_realtime_json))
        .with_state(gtfs_state)
        .layer(CompressionLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Spawn a listener that does not use the SQLite database
fn spawn_listener<Fut: Future + Send + 'static>(config: &Arc<BBConfig>, listener_type: GTFSResponder, tx: &Sender<GTFSResponse>, listener: fn(Sender<GTFSResponse>, Arc<BBConfig>) -> Fut) {
    if config.is_enabled(listener_type) {
        let sender = tx.clone();
        let cfg = Arc::clone(config);
        tokio::task::spawn(async move {
            listener(sender, cfg).await;
        });
        info!("{} {}", listener_type, Green.paint("enabled"));
    } else {
        info!("{} {}", listener_type, Red.paint("disabled"));
    }
}

/// Spawn a listener that uses the SQLite database
fn spawn_listener_db<Fut: Future + Send + 'static>(config: &Arc<BBConfig>, listener_type: GTFSResponder, tx: &Sender<GTFSResponse>, db: &Arc<DBPool>, listener: fn(Sender<GTFSResponse>, Arc<BBConfig>, db: Arc<DBPool>) -> Fut) {
    if config.is_enabled(listener_type) {
        let sender = tx.clone();
        let cfg = Arc::clone(config);
        let db_arc = Arc::clone(db);
        tokio::task::spawn(async move {
            listener(sender, cfg, db_arc).await;
        });
        info!("{} {}", listener_type, Green.paint("enabled"));
    } else {
        info!("{} {}", listener_type, Red.paint("disabled"));
    }
}

/// Create a GTFS feed message out of each feed provider
fn generate_gtfs_message(state_lock: &Arc<GTFSState>) -> FeedMessage {
    let mut feed_msg: FeedMessage = Default::default();
    feed_msg.header.gtfs_realtime_version = "2.0".parse().unwrap();
    feed_msg.header.timestamp = Some(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    {
        let vehicles = state_lock.vehicles.read().unwrap();
        let alerts = state_lock.alerts.read().unwrap();
        let alert_entities: Vec<Vec<FeedEntity>> = alerts.values().map(|alerts| {
           return alerts.iter().map(|alert| {
               FeedEntity {
                   id: "".to_string(),
                   is_deleted: None,
                   trip_update: None,
                   vehicle: None,
                   alert: Some(alert.clone()),
                   shape: None,
               }
           }).collect()
        }).collect();
        feed_msg.entity = vehicles.values().chain(alert_entities.iter()).flat_map(|fe| fe.to_vec()).collect();
    }

    feed_msg
}

/// Export the GTFS feed message in the Protobuf format
async fn gtfs_realtime_proto(State(state_lock): State<Arc<GTFSState>>) -> Vec<u8> {
    generate_gtfs_message(&state_lock).encode_to_vec()
}

/// Export the GTFS feed message as JSON
async fn gtfs_realtime_json(State(state_lock): State<Arc<GTFSState>>) -> String {
    serde_json::to_string(&generate_gtfs_message(&state_lock)).unwrap_or("".parse().unwrap())
}