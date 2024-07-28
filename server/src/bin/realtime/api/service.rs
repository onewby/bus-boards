use std::cmp::max;
use std::collections::HashMap;
use std::str;
use std::sync::{Arc, RwLock};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::ErrorResponse;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, Timelike, Utc};
use geo::{GeodesicDistance, GeodesicLength, HaversineClosestPoint, HaversineDistance, LineInterpolatePoint, LineLocatePoint};
use geo_types::{coord, Line, Point};
use itertools::Itertools;
use regex::Regex;
use serde_nested_with::serde_nested;

use crate::{GTFSAlerts, GTFSState, uw};
use crate::api::util;
use crate::api::util::{cache_service_data, INTERNAL_ERROR, ServiceError};
use crate::db::{get_service_shape, get_stop_positions, OperatorsQuery, query_service, query_service_operator, query_stops, StopsQuery};
use crate::transit_realtime::{FeedEntity, Position, TranslatedString, TripUpdate};
use crate::transit_realtime::trip_descriptor::ScheduleRelationship::Canceled;
use crate::transit_realtime::trip_update::stop_time_update::ScheduleRelationship::Skipped;
use crate::util::{adjust_timestamp, haversine_closest_point};

pub async fn get_service(Query(params): Query<HashMap<String, String>>, State(state): State<Arc<GTFSState>>) -> Result<Json<ServiceData>, ErrorResponse> {
    let id = params.get("id").or_error((StatusCode::BAD_REQUEST, "ID not provided"))?;
    let data = get_service_data(&state, id)?;
    Ok(Json(data))
}

pub fn get_service_data(state: &Arc<GTFSState>, id: &String) -> Result<ServiceData, ErrorResponse> {
    let service = query_service(&state.db, id).or_error((StatusCode::BAD_REQUEST, "Service not found"))?;
    let mut stops = query_stops(&state.db, id).or_error(INTERNAL_ERROR)?;
    let operator = query_service_operator(&state.db, id).or_error(INTERNAL_ERROR)?;
    let shape = get_service_shape(&state.db, id);

    // Better coach listings - show root locality name
    if state.operators.intercity_operators.contains(&operator.name) {
        stops.iter_mut().for_each(|stop| {
            fix_stop_name(stop);
        })
    }
    // Simplify tram listings - show more akin to trains
    simplify_tram_names(&mut stops, &operator, &state.operators);

    let route = shape.unwrap_or_else(|_| polyline::encode_coordinates(
        stops.iter().map(|s| coord! {x: s.long, y: s.lat}), 5).unwrap());

    let mut cancelled = false;
    let find_realtime_trip = util::find_realtime_trip_with_gtfs(id, &state.vehicles);
    let realtime = if let Some((_, ref trip)) = find_realtime_trip {
        cancelled = uw! {trip.vehicle.as_ref()?.trip.as_ref()?.schedule_relationship} == Some(Canceled.into())
            || uw! {trip.trip_update.as_ref()?.trip.schedule_relationship} == Some(Canceled.into());
        if cancelled {
            stops.iter_mut().for_each(|stop| stop.status = Some("Cancelled".to_string()))
        }

        let current_stop_seq = uw! {trip.vehicle.as_ref()?.current_stop_sequence.clone()};
        let current_pos = uw! {trip.vehicle.as_ref()?.position.clone()};
        let time_now = Utc::now();

        if let Some(ref trip_update) = trip.trip_update {
            Some(realtime_from_trip_update(&mut stops, &trip, current_pos, &time_now, &trip_update))
        } else if current_stop_seq.is_some() && current_pos.is_some() {
            realtime_from_position(&state, id, &mut stops, &trip, cancelled, current_stop_seq, current_pos, &time_now)
        } else {
            None
        }
    } else {
        None
    };

    let alerts = get_alerts(&state.alerts, Some(id), Some(&service.route_id), Some(&operator.id));

    let data = ServiceData {
        service: ServiceInfo {
            code: service.code,
            dest: service.dest.to_string(),
            cancelled
        },
        operator,
        branches: vec![
            ServiceBranch {
                dest: service.dest,
                stops,
                realtime,
                route,
            }
        ],
        alerts
    };
    
    if let Some((resp, _)) = find_realtime_trip.as_ref() {
        cache_service_data(&state.realtime_cache, resp.clone(), id.as_str(), &data);
    }
    
    Ok(data)
}

pub fn get_alerts(alerts: &RwLock<GTFSAlerts>, trip_id: Option<&String>, route_id: Option<&String>, agency_id: Option<&String>) -> Vec<StopAlert> {
    alerts.read().unwrap().values().flatten().filter(|alert| {
        alert.informed_entity.iter().any(|entity|
            (entity.agency_id.as_ref() == agency_id && agency_id.is_some())
            || (entity.route_id.as_ref() == route_id && route_id.is_some())
            || (uw!(entity.trip.as_ref()?.trip_id.as_ref()) == trip_id && trip_id.is_some())
        )
    }).map(|alert| {
        StopAlert {
            header: find_best_match(&alert.header_text),
            description: find_best_match(&alert.description_text),
            url: find_best_match(&alert.url)
        }
    }).collect_vec()
}

pub(crate) fn find_best_match(str: &Option<TranslatedString>) -> Option<String> {
    str.as_ref().map(|str| {
        str.translation.iter().find(|t| t.language == Some("en".to_string())).map(|t| t.text.clone()).unwrap_or_else(|| str.translation[0].text.to_string())
    })
}

fn realtime_from_position(state: &Arc<GTFSState>, id: &String, stops: &mut Vec<StopsQuery>, trip: &FeedEntity, cancelled: bool, current_stop_seq: Option<u32>, current_pos: Option<Position>, time_now: &DateTime<Utc>) -> Option<RealtimeInfo> {
    let current_stop = current_stop_seq.unwrap();
    let current_pos = current_pos.unwrap();
    let current_stop_index = stops.iter().find_position(|stop| stop.seq as u32 == current_stop);
    let pos = get_stop_positions(&state.db, current_stop, id);

    if pos.len() == 2 && current_stop_index.is_some() {
        // Positioning
        let current_stop_index = current_stop_index.unwrap().0;
        let prev_curr = Line::new(pos[0].pos, pos[1].pos);
        let current_pos_point = Point::from(coord! {x: current_pos.longitude as f64, y: current_pos.latitude as f64});
        let line_point = haversine_closest_point(&prev_curr, &current_pos_point);
        let pct = line_point.geodesic_distance(&Point::from(prev_curr.start)) / prev_curr.geodesic_length();
        let pct = if pct.is_nan() { 1.0 } else { pct };
        
        if !cancelled {
            let date = get_start_date(&trip, &time_now);
            let scheduled_times = stops.iter().map(|stop| {
                ScheduledTime {
                    arr: date + stop.arr,
                    dep: date + stop.dep
                }
            }).collect_vec();

            let prev_stop = &scheduled_times[(current_stop_index - 1).max(0)];
            let curr_stop = &scheduled_times[current_stop_index];

            // Get the time that the bus should have been at this position at
            let expected_time = prev_stop.dep + TimeDelta::milliseconds(((curr_stop.arr - prev_stop.dep).num_milliseconds() as f64 * pct) as i64);
            let vehicle_time = adjust_timestamp(&uw!(trip.vehicle.as_ref()?.timestamp).and_then(|t| DateTime::from_timestamp(t as i64, 0)).unwrap_or(*time_now));
            let mut delay = vehicle_time - expected_time;

            // Apply delay to all stops past the current stop
            // (don't show 'Departed' if too close to the last stop - may be a GPS error)
            let evaluate_index = max(current_stop_index - 1, 0);
            let include_last_stop = if Point::from(stops[evaluate_index].position()).geodesic_distance(&current_pos_point) <= 50.0 { 1 } else { 0 };

            // Only show Departed 5 mins before departure time
            for i in 0..stops.len() {
                let scheduled_time = &scheduled_times[i];
                let delayed_time = scheduled_time.arr + delay;
                
                if i < current_stop_index - include_last_stop && (scheduled_time.dep - adjust_timestamp(time_now)).num_seconds() < 120 {
                    stops[i].status = Some("Departed".to_string());
                    continue;
                }

                if delay >= TimeDelta::milliseconds(1000 * 120) || delay <= TimeDelta::milliseconds(-1000 * 60) {
                    stops[i].status = if scheduled_time.dep.minute() == delayed_time.minute() {
                        Some("On time".to_string())
                    } else {
                        Some(format!("Exp. {}", delayed_time.format("%H:%M")))
                    };

                    // Absorb delay in longer layovers
                    delay = delay - (scheduled_time.dep - scheduled_time.arr);
                    if delay.num_milliseconds() < 0 {
                        delay = TimeDelta::default();
                        stops[i].status = Some("On time".to_string());
                    }
                } else {
                    stops[i].status = Some("On time".to_string());
                }

                // Show current delayed stop in major stops list for context (since previous stops don't show delay, can look on time when delayed)
                if stops[current_stop_index].status != Some("On time".to_string()) {
                    stops[current_stop_index].major = true
                }
            };

            Some(RealtimeInfo {
                stop: current_stop_index as i64,
                pct,
                pos: Some(current_pos),
            })
        } else {
            None
        }
    } else {
        Some(RealtimeInfo {
            stop: -1,
            pct: 0.0,
            pos: Some(current_pos),
        })
    }
}

fn realtime_from_trip_update(stops: &mut Vec<StopsQuery>, trip: &FeedEntity, current_pos: Option<Position>, time_now: &DateTime<Utc>, trip_update: &TripUpdate) -> RealtimeInfo {
    let date = get_start_date(trip, time_now);
    let scheduled_times: Vec<DateTime<Utc>> = stops.iter().map(|stop| date + stop.dep).collect_vec();
    let mut actual_times = stops.iter().enumerate().map(|(i, stop)| {
        let update = trip_update.stop_time_update.iter()
            .find(|stu| Some(stop.seq as u32) == stu.stop_sequence);
        if let Some(update) = update {
            if update.schedule_relationship == Some(Skipped.into()) {
                return None;
            }
            let time = update.departure.as_ref().or(update.arrival.as_ref());
            if let Some(time) = uw!(time?.time) {
                return DateTime::from_timestamp(time, 0);
            }
        }
        return Some(scheduled_times[i]);
    }).collect_vec();

    actual_times.iter_mut().enumerate().for_each(|(i, actual_time)| {
        if actual_time.is_none() {
            stops[i].status = Some("Skipped".to_string());
            *actual_time = Some(scheduled_times[i]);
        } else if (actual_time.unwrap() - scheduled_times[i]).num_milliseconds() < 60 * 1000 {
            stops[i].status = Some("On time".to_string())
        } else {
            stops[i].status = Some(format!("Exp. {}", actual_time.unwrap().format("%H:&M")))
        }
    });
    let actual_times = actual_times.into_iter().filter_map(|x| x).collect_vec();
    assert_eq!(scheduled_times.len(), actual_times.len());

    let current = actual_times.iter().find_position(|t| t >= &&time_now).map(|c| c.0).unwrap_or(0);
    let pct = match current {
        0 => 0.0,
        i => (actual_times[i] - time_now).num_milliseconds() as f64 / (actual_times[i] - actual_times[i - 1]).num_milliseconds() as f64,
    };

    stops.iter_mut().enumerate().take(current).for_each(|(i, stop)| {
        stop.status = Some(format!("Dep. {}", actual_times[i].format("%H:%M")));
    });

    RealtimeInfo {
        stop: current as i64,
        pct,
        pos: current_pos
    }
}

fn get_start_date(trip: &FeedEntity, time_now: &DateTime<Utc>) -> DateTime<Utc> {
    uw!(trip.vehicle.as_ref()?.trip.as_ref()?.start_date.as_ref()).map(|d| NaiveDate::parse_from_str(d.as_str(), "%Y%m%d").map(|dt| dt.and_time(NaiveTime::default()).and_utc()).ok())
        .flatten().unwrap_or(*time_now)
}

fn simplify_tram_names(stops: &mut Vec<StopsQuery>, operator: &OperatorsQuery, operators: &OperatorColours) {
    match operator.name.as_str() {
        "Edinburgh Trams" | "Tyne & Wear Metro" | "Metrolink" | "SPT Subway" | "London Underground (TfL)" =>
            stops.iter_mut().for_each(|stop| {
                stop.ind = None;
                stop.display_name = operators.suffixes[&operator.name].replace(stop.name.as_str(), "").to_string();
                if stop.name != "Rail Station" { stop.loc = None; }
            }),
        "West Midlands Metro" | "Nottingham Express Transit (Tram)" | "London Docklands Light Railway - TfL"
        | "London Tramlink" =>
            stops.iter_mut().for_each(|stop| {
                stop.display_name = operators.suffixes[&operator.name].replace(stop.name.as_str(), "").to_string();
                if stop.name != "Rail Station" { stop.loc = None; }
            }),
        "South Yorkshire Future Tram" =>
            stops.iter_mut().for_each(|stop| {
                if stop.name != "Rail Station" { stop.loc = None; }
            }),
        _ => {}
    }
}

fn fix_stop_name(stop: &mut StopsQuery) {
    if stop.loc.is_some() && (stop.loc.as_ref().unwrap().contains("University") || stop.loc.as_ref().unwrap().contains("Airport")) { return; }
    let empty_str = String::default();
    let existing_loc = stop.loc.as_ref().unwrap_or(&empty_str).clone();
    stop.loc = Some(stop.full_loc.split(" › ").next().map(str::to_string).unwrap_or(stop.full_loc.to_string()));
    if (stop.name == "Park and Ride" || stop.name == "Rail Station") && Some(existing_loc.clone()) != stop.loc {
        stop.display_name = format!("{existing_loc} {}", stop.name)
    } else if stop.loc == Some("Centenary Square".to_string()) {
        stop.loc = Some("Birmingham".to_string())
    }
}

#[serde_nested]
#[derive(Serialize, Deserialize)]
pub struct OperatorColours {
    pub operator_matches: HashMap<String, String>,
    pub route_overrides: HashMap<String, HashMap<String, String>>,
    pub route_overrides_prefixes: HashMap<String, HashMap<String, String>>,
    #[serde_nested(sub="Regex", serde(with="serde_regex"))]
    pub operator_regex: Vec<(Regex, String)>,
    pub intercity_operators: Vec<String>,
    #[serde_nested(sub="Regex", serde(with="serde_regex"))]
    pub suffixes: HashMap<String, Regex>
}

#[derive(Serialize, Clone)]
pub struct RealtimeInfo {
    stop: i64,
    pct: f64,
    pos: Option<Position>
}

pub struct ScheduledTime {
    arr: DateTime<Utc>,
    dep: DateTime<Utc>
}

#[derive(Serialize, Clone)]
pub struct StopAlert {
    pub(crate) header: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) url: Option<String>
}

#[derive(Serialize, Clone)]
pub struct ServiceInfo {
    pub code: String,
    pub dest: String,
    pub cancelled: bool
}

#[derive(Serialize, Clone)]
pub struct ServiceBranch {
    pub dest: String,
    pub stops: Vec<StopsQuery>,
    pub realtime: Option<RealtimeInfo>,
    pub route: String
}

#[derive(Serialize, Clone)]
pub struct ServiceData {
    pub service: ServiceInfo,
    pub operator: OperatorsQuery,
    pub branches: Vec<ServiceBranch>,
    pub alerts: Vec<StopAlert>
}