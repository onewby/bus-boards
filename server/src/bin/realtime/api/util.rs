use std::collections::HashMap;
use std::str;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{ErrorResponse, IntoResponse, Response};
use axum::Json;
use strum::IntoEnumIterator;
use BusBoardsServer::GTFSResponder;

use crate::api::service::{get_service_data, ServiceData};
use crate::transit_realtime::FeedEntity;
use crate::{GTFSState, GTFSVehicles, RealtimeCache};

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

pub fn find_realtime_trip(id: &str, vehicles: &GTFSVehicles) -> Option<FeedEntity> {
    vehicles.pin().iter().find_map(|(_, v)| {
        v.get(id).cloned()
    })
}

pub fn find_realtime_trip_with_gtfs(id: &str, vehicles: &GTFSVehicles) -> Option<(GTFSResponder, FeedEntity)> {
    vehicles.pin().iter().find_map(|(resp, v)| {
        v.get(id).map(|entity| (resp.clone(), entity.clone()))
    })
}

pub fn get_or_cache_service_data<'a>(state: &Arc<GTFSState>, responder: GTFSResponder, trip_id: &str) -> Option<ServiceData> {
    let result = state.realtime_cache.pin().get(&responder).and_then(|x| x.pin().get(trip_id).map(|x| x.clone()));
    if result.is_some() {
        result
    } else {
        get_service_data(state, &trip_id.to_string()).ok()
    }
}

pub fn get_or_cache_all_service_data<'a>(state: &Arc<GTFSState>, trip_id: &str) -> Option<ServiceData> {
    state.realtime_cache.pin().iter().find_map(|(_, x)| {
        x.pin().get(trip_id).map(|x| x.clone())
    }).or_else(|| {
        get_service_data(state, &trip_id.to_string()).ok()
    })
}

pub fn cache_service_data(cache: &Arc<RealtimeCache>, responder: GTFSResponder, trip_id: &str, service_data: &ServiceData) {
    cache.pin().get_or_insert_with(responder, || papaya::HashMap::new()).pin().insert(trip_id.to_string(), service_data.clone());
}

pub fn map_feed_entities(fe: &Vec<FeedEntity>) -> HashMap<String, FeedEntity> {
    let mut map = HashMap::with_capacity(fe.len());
    for entity in fe {
        if let Some(vehicle) = &entity.vehicle {
            if let Some(trip) = &vehicle.trip {
                if let Some(trip_id) = &trip.trip_id {
                    map.insert(trip_id.clone(), entity.clone());
                }
            }
        }
    }
    map
}