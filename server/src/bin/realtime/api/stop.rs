use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{ErrorResponse, IntoResponse};
use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeDelta, Timelike, Utc};
use futures::{stream, StreamExt};
use itertools::Itertools;
use memoize::lazy_static::lazy_static;
use polars::export::arrow::io::iterator::StreamingIterator;
use polars::export::rayon::iter::IntoParallelRefMutIterator;
use polars::export::rayon::iter::ParallelIterator;
use regex::Regex;
use tokio::task::JoinHandle;

use BusBoardsServer::GTFSResponder;

use crate::{GTFSAlerts, GTFSState, uw};
use crate::api::darwin::{GetDepartureBoardRequest, GetDepartureBoardResponse, LDBService, SoapFault, StationBoard};
use crate::api::service::{find_best_match, StopAlert};
use crate::api::util::{find_realtime_trip_with_gtfs, get_or_cache_service_data, INTERNAL_ERROR, ServiceError, get_or_cache_all_service_data};
use crate::db::{get_services_between, get_stance_info, get_stop_info, StanceInfo, StopInfoQuery, StopService};
use crate::transit_realtime::FeedEntity;
use crate::util::adjust_timestamp;

pub async fn get_stop(Query(params): Query<HashMap<String, String>>, State(state): State<Arc<GTFSState>>) -> Result<Json<StopResponse>, ErrorResponse> {
    let locality = params.get("locality").or_error(INVALID_QUERY)?;
    let name = params.get("name").or_error(INVALID_QUERY)?;
    if name == "" || locality == "" { return Err(ErrorResponse::from(INVALID_QUERY.into_response())) }
    let date = match params.get("date") {
        None => adjust_timestamp(&Utc::now()).with_second(0).unwrap().with_nanosecond(0).unwrap(),
        Some(date_str) => NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M").map(|t| t.and_utc())
            .or_error((StatusCode::BAD_REQUEST, "Invalid date"))?,
    };
    let filter_loc = params.get("filterLoc");
    let filter_name = params.get("filterName");
    
    let data = get_stop_data(&state, locality, name, &date, filter_loc, filter_name).await?;
    Ok(Json(data))
}

pub async fn get_stop_data(state: &Arc<GTFSState>, locality: &String, name: &String, date: &DateTime<Utc>, filter_loc: Option<&String>, filter_name: Option<&String>) -> Result<StopResponse, ErrorResponse> {
    let filter = filter_loc.is_some() && filter_name.is_some();

    let start_time = *date - TimeDelta::hours(2);
    let end_time = *date + TimeDelta::hours(2);
    let offset = ((*date - adjust_timestamp(&Utc::now())).num_seconds() as f32 / 60.0).ceil();

    let stop_info = get_stop_info(&state.db, name, locality).or_error((StatusCode::NOT_FOUND, "Stop not found"))?;
    let mut stance_info = get_stance_info(&state.db, stop_info.id).or_error(INTERNAL_ERROR)?;

    // Get stop alerts
    let alerts = stance_info.iter()
        .flat_map(|stance| get_stop_alerts(&state.alerts, stance.code.as_str()))
        .collect_vec();
    
    // Sort by indicator
    stance_info.iter_mut().for_each(|stance| {
        if let None = stance.indicator {
            stance.indicator = Some("".to_string());
        }
    });
    stance_info.sort_by(|a, b| a.indicator.as_ref().unwrap().to_ascii_lowercase().cmp(&b.indicator.as_ref().unwrap().to_ascii_lowercase()));

    // Get station results
    let crs = stance_info.iter().filter_map(|stance| stance.crs.clone()).unique().collect_vec();
    let stations: Option<JoinHandle<Vec<StationBoard>>> = if crs.len() > 0 {
        Some(tokio::spawn(async move { get_station_departures(offset, crs).await }))
    } else {
        None
    };

    // Get actual service list
    let mut services = get_services_between(&state.db, &start_time, &end_time, stop_info.id, filter, filter_name, filter_loc).or_error(INTERNAL_ERROR)?;

    // Coastliner/Flyer workaround (duplicate services under Coastliner + Flyer names, only Coastliner ones track)
    let mut replaced: Vec<String> = vec![];
    for i in 0..services.len() {
        if services[i].operator_name == "Coastliner" && services[i].route_short_name.starts_with("A") {
            let to_replace = services.iter().find(
                |s2| s2.operator_name == "Flyer" && s2.route_short_name == services[i].route_short_name && s2.departure_time == services[i].departure_time);
            if to_replace.is_some() {
                replaced.push(to_replace.unwrap().trip_id.clone());
                services[i].operator_name = "Flyer".to_string();
            }
        }
    }
    // Remove Flyer duplicates, and an SPT Subway duplicate entry workaround
    services.retain(|service| replaced.iter().find(|s| service.trip_id == **s).is_none()
        && (service.operator_name != "SPT Subway" || service.indicator.len() > 0));

    // Get delay status (On time, Exp. XX:XX)
    let mut tracking_stops = services.iter_mut()
        .map(|stop|
            {
                let trip_id = stop.trip_id.clone();
                (stop, find_realtime_trip_with_gtfs(trip_id.as_str(), &state.vehicles)
                    .map(|(r, fe)| r))
            })
        .collect_vec();
    tracking_stops.par_iter_mut().for_each(|stop| set_status_by_realtime(state, stop));
    services.retain(|stop| stop.departure_time[0] >= *date ||
        stop.status.as_ref().map(|status| status.starts_with("Exp. ") || status == "Cancelled").unwrap_or(false));

    // Add train times if applicable
    let stations = match stations {
        None => vec![],
        Some(stations) => stations.await.unwrap_or_else(|_| vec![])
    };
    let mut train_times = stations.iter().flat_map(|s| s.train_services.iter().cloned())
        .filter(|service| service.operator_code != "TW")
        .map(|service| {
            let dep_time = NaiveTime::parse_from_str(&service.std.or(service.sta).unwrap(), "%H:%M").unwrap();
            let dep_datetime = if dep_time.hour() < date.hour() {
                (date.date_naive() + TimeDelta::days(1)).and_time(dep_time).and_utc()
            } else {
                date.date_naive().and_time(dep_time).and_utc()
            };
            StopService {
                trip_id: service.service_id,
                trip_headsign: service.destination.locations.iter().map(|loc| loc.location_name.clone()).join(" & "),
                departure_time: vec![dep_datetime],
                indicator: vec![service.platform.map(|p| format!("Platform {p}").to_string()).unwrap_or("Platform TBC".to_string())],
                route_short_name: "".to_string(),
                operator_id: service.operator_code,
                operator_name: service.operator,
                stop_sequence: 0,
                _type: "train".to_string(),
                colour: "#777".to_string(),
                status: service.etd.clone()
                    .take_if(|etd| etd.chars().next().unwrap().is_numeric())
                    .map(|etd| format!("Exp. {etd}")).or(service.etd.as_ref().map(|etd| etd.to_string())),
                then_headsign: None,
            }
        }).collect_vec();

    services.append(&mut train_times);
    services.sort_by_key(|service| service.departure_time[0]);

    // Set agency colours
    let agencies: HashSet<String> = services.iter().map(|time| time.operator_name.clone()).collect();
    let colours: HashMap<String, String> = agencies.iter().map(|a| {
        (a.to_string(), state.operators.operator_matches.get(a).cloned()
            .or_else(|| state.operators.operator_regex.iter().find(|(regex, colour)| regex.find(a.as_str()).is_some())
                .map(|(regex, colour)| colour.to_string()))
            .unwrap_or("#777".to_string()))
    }).collect();
    services.iter_mut().for_each(|time| {
        time.colour = uw!(state.operators.route_overrides.get(&time.operator_name)?.get(&time.route_short_name))
            .or(uw!(state.operators.route_overrides_prefixes.get(&time.operator_name)?.get(
                PREFIX_REGEX.captures(time.route_short_name.as_str()).and_then(|cap| Some(cap.get(1)?.as_str())).unwrap_or(""))))
            .or(colours.get(&time.operator_name))
            .unwrap().to_string();
    });

    // Merge consecutive stops
    let mut i = 0;
    while i < services.len() {
        let mut duplicates = vec![];
        for j in i+1..services.len() {
            if services[i].trip_id == services[j].trip_id {
                if services[*duplicates.last().unwrap_or(&i)].stop_sequence + 1 == services[j].stop_sequence {
                    duplicates.push(j);
                } else {
                    break;
                }
            }
        }

        duplicates.iter().for_each(|&service| {
            let dep_time = services[service].departure_time[0].clone();
            let indicator = services[service].indicator[0].clone();
            services[i].departure_time.push(dep_time);
            services[i].indicator.push(indicator);
        });
        duplicates.iter().rev().for_each(|&service| {
            services.remove(service);
        });
        i += 1;
    }

    Ok(StopResponse {
        stop: stop_info,
        stances: stance_info,
        times: services,
        alerts,
    })
}

fn set_status_by_realtime(state: &Arc<GTFSState>, (stop, resp): &mut (&mut StopService, Option<GTFSResponder>)) {
    let service_data = match resp {
        None => get_or_cache_all_service_data(state, stop.trip_id.as_str()),
        Some(resp) => get_or_cache_service_data(&state, *resp, stop.trip_id.as_str())
    };
    if let Some(service) = service_data {
        if service.branches.len() != 1 {
            return;
        }
        stop.status = service.branches[0].stops.iter().find(|ss| ss.seq == stop.stop_sequence).and_then(|s| s.status.clone())
    }
}

async fn get_station_departures(offset: f32, crs: Vec<String>) -> Vec<StationBoard> {
    if offset.abs() <= 120.0 {
        let departure_boards: Vec<StationBoard> = stream::iter(crs)
            .filter_map(|crs| async move {
                get_station(crs, offset as i32).await.ok().and_then(|board| {
                    board.response
                })
            })
            .collect().await;
        departure_boards
    } else {
        vec![]
    }
}

fn get_stop_alerts(cache: &RwLock<GTFSAlerts>, code: &str) -> Vec<StopAlert> {
    cache.read().unwrap().values().flatten()
        .filter(|alert| alert.informed_entity.iter().any(|entity| {
            match entity.stop_id.as_ref() {
                None => false,
                Some(stop_id) => stop_id == code
            }
        }))
        .map(|alert| {
            StopAlert {
                header: find_best_match(&alert.header_text),
                description: find_best_match(&alert.description_text),
                url: find_best_match(&alert.url)
            }
        }).collect_vec()
}

pub async fn get_station(crs: String, offset: i32) -> Result<GetDepartureBoardResponse, Option<SoapFault>> {
    let ldb = LDBService::new(std::env::var("DARWIN_API_KEY").unwrap_or("".to_string()));
    ldb.get_departure_board(GetDepartureBoardRequest {
        num_rows: 150,
        crs,
        filter_crs: None,
        filter_type: None,
        time_offset: Some(offset),
        time_window: None,
    }).await
}

#[derive(Serialize)]
pub struct StopResponse {
    stop: StopInfoQuery,
    stances: Vec<StanceInfo>,
    times: Vec<StopService>,
    alerts: Vec<StopAlert>
}

lazy_static! {
    static ref PREFIX_REGEX: Regex = Regex::new("(.*)[A-Z]").unwrap();
}

const INVALID_QUERY: (StatusCode, &str) = (StatusCode::BAD_REQUEST, "Invalid query provided.");