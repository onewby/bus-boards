mod bods;
mod disruptions;
mod db;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate yaserde_derive;

use std::collections::HashMap;
use std::future::Future;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime};
use axum::extract::State;
use axum::Router;
use axum::routing::{get};
use bytes::Buf;
use prost::Message;
use tokio::sync::{mpsc};
use tokio::sync::mpsc::{Sender};
use crate::bods::bods_listener;
use crate::db::{DBPool, open_db};
use crate::disruptions::disruptions_listener;
use crate::transit_realtime::{Alert, FeedEntity, FeedMessage};

pub mod transit_realtime {
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

#[derive(Copy, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub enum GTFSResponder {
    BODS, DISRUPTIONS
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
    let gtfs_state = Arc::new(GTFSState::default());
    let (tx, mut rx) = mpsc::channel::<GTFSResponse>(16);
    let gtfs_ref = gtfs_state.clone();
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            gtfs_ref.vehicles.write().unwrap().insert(response.0, response.1);
            gtfs_ref.alerts.write().unwrap().insert(response.0, response.2);
        }
    });
    let db = open_db();
    let arc_db = Arc::new(db);
    spawn_listener(&tx, bods_listener);
    spawn_listener_db(&tx, &arc_db.clone(), disruptions_listener);

    // build our application with a route
    let app = Router::new()
        .route("/api/gtfsrt/proto", get(gtfs_realtime_proto))
        .route("/api/gtfsrt/json", get(gtfs_realtime_json))
        .with_state(gtfs_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn spawn_listener<Fut: Future + Send + 'static>(tx: &Sender<GTFSResponse>, listener: fn(Sender<GTFSResponse>) -> Fut) {
    let sender = tx.clone();
    tokio::task::spawn(async move {
        listener(sender).await;
    });
}

fn spawn_listener_db<'a, Fut: Future + Send + 'static>(tx: &Sender<GTFSResponse>, db: &Arc<DBPool>, listener: fn(Sender<GTFSResponse>, db: Arc<DBPool>) -> Fut) {
    let sender = tx.clone();
    let db_arc = Arc::clone(db);
    tokio::task::spawn(async move {
        listener(sender, db_arc).await;
    });
}

fn generate_gtfs_message(state_lock: &Arc<GTFSState>) -> FeedMessage {
    let mut feed_msg: FeedMessage = Default::default();
    feed_msg.header.gtfs_realtime_version = "2.0".parse().unwrap();
    feed_msg.header.timestamp = Some(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    {
        let vehicles = state_lock.vehicles.read().unwrap();
        feed_msg.entity = vehicles.values().flat_map(|fe| fe.to_vec()).collect();
    }

    feed_msg
}

async fn gtfs_realtime_proto(State(state_lock): State<Arc<GTFSState>>) -> Vec<u8> {
    generate_gtfs_message(&state_lock).encode_to_vec()
}

async fn gtfs_realtime_json(State(state_lock): State<Arc<GTFSState>>) -> String {
    serde_json::to_string(&generate_gtfs_message(&state_lock)).unwrap_or("".parse().unwrap())
}