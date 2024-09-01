use std::str;
use std::sync::{Arc, RwLock};

use axum::http::StatusCode;
use axum::Json;
use axum::response::{ErrorResponse, IntoResponse, Response};
use dashmap::DashMap;
use strum::IntoEnumIterator;
use BusBoardsServer::GTFSResponder;

use crate::{GTFSState, GTFSVehicles, RealtimeCache};
use crate::api::service::{get_service_data, ServiceData};
use crate::transit_realtime::FeedEntity;

fn error(code: StatusCode, msg: &str) -> Response {
    (code, msg.to_string()).into_response()
}

pub const INTERNAL_ERROR: (StatusCode, &str) = (StatusCode::INTERNAL_SERVER_ERROR, "An internal error has occurred.");

pub trait ServiceError<T> {
    fn or_error(self, error: (StatusCode, &str)) -> Result<T, ErrorResponse>;
}

impl <T, E> ServiceError<T> for Result<T, E> {
    fn or_error(self, (code, msg): (StatusCode, &str)) -> Result<T, ErrorResponse> {
        self.map_err(|_| (code, Json(JsonError::from(msg))).into_response().into())
    }
}

impl <T> ServiceError<T> for Option<T> {
    fn or_error(self, (code, msg): (StatusCode, &str)) -> Result<T, ErrorResponse> {
        match self {
            None => Err((code, Json(JsonError::from(msg))).into_response().into()),
            Some(obj) => Ok(obj)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonError {
    pub message: String
}

impl JsonError {
    pub fn from<T>(msg: T) -> JsonError where T: ToString {
        JsonError {
            message: msg.to_string()
        }
    }
}

#[macro_export]
macro_rules! uw {
    ( $x:expr ) => {
        (|| $x)()
    };
}

pub fn find_realtime_trip(id: &str, vehicles: &RwLock<GTFSVehicles>) -> Option<FeedEntity> {
    let vehicles = vehicles.read().unwrap();
    vehicles.values().flatten().find_map(|entity| {
        entity.vehicle.as_ref()?.trip.as_ref()?.trip_id.as_ref().filter(|tid| tid == &id).map(|_| entity.clone())
    })
}

pub fn find_realtime_trip_with_gtfs(id: &str, vehicles: &RwLock<GTFSVehicles>) -> Option<(GTFSResponder, FeedEntity)> {
    let vehicles = vehicles.read().unwrap();
    vehicles.iter().flat_map(|(resp, v)| v.iter().map(move |fe| (resp, fe)))
        .find_map(|(resp, entity)| {
            entity.vehicle.as_ref()?.trip.as_ref()?.trip_id.as_ref().filter(|tid| tid == &id).map(|_| (resp.clone(), entity.clone()))
        })
}

pub fn get_or_cache_service_data<'a>(state: &Arc<GTFSState>, responder: GTFSResponder, trip_id: &str) -> Option<ServiceData> {
    let result = state.realtime_cache.get(&responder).and_then(|x| x.get(trip_id).map(|x| x.clone()));
    if result.is_some() {
        result
    } else {
        get_service_data(state, &trip_id.to_string()).ok()
    }
}

pub fn get_or_cache_all_service_data<'a>(state: &Arc<GTFSState>, trip_id: &str) -> Option<ServiceData> {
    state.realtime_cache.iter().find_map(|x| {
        x.get(trip_id).map(|x| x.clone())
    }).or_else(|| {
        get_service_data(state, &trip_id.to_string()).ok()
    })
}

pub fn cache_service_data(cache: &Arc<RealtimeCache>, responder: GTFSResponder, trip_id: &str, service_data: &ServiceData) {
    cache.entry(responder).or_insert_with(|| DashMap::new()).insert(trip_id.to_string(), service_data.clone());
}
