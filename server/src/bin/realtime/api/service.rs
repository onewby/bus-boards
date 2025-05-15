use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::ops::Add;
use std::str;
use std::sync::{Arc, RwLock};

use crate::api::util;
use crate::api::util::{cache_service_data, get_or_cache_all_service_data, ServiceError, INTERNAL_ERROR};
use crate::db::{find_links, get_service_shape, get_stop_positions, query_service, query_service_operator, query_stops, Connections, OperatorsQuery, StopsQuery};
use crate::transit_realtime::trip_descriptor::ScheduleRelationship::Canceled;
use crate::transit_realtime::trip_update::stop_time_update::ScheduleRelationship::Skipped;
use crate::transit_realtime::{FeedEntity, Position, TranslatedString, TripUpdate};
use crate::util::{adjust_timestamp, haversine_closest_point};
use crate::{uw, GTFSAlerts, GTFSState};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::ErrorResponse;
use axum::Json;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, Timelike, Utc};
use geo::{GeodesicDistance, GeodesicLength, HaversineClosestPoint, HaversineDistance, LineInterpolatePoint, LineLocatePoint};
use geo_types::{coord, Line, Point};
use itertools::Itertools;
use polars::export::arrow::temporal_conversions::MILLISECONDS_IN_DAY;
use regex::Regex;
use serde_nested_with::serde_nested;
use util::find_realtime_trip_with_gtfs;

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
    let links = find_links(&state.db, id);

    // Better coach listings - show root locality name
    if state.operators.intercity_operators.contains(&operator.name) {
        stops.iter_mut().for_each(|stop| {
            fix_stop_name(stop);
        })
    }
    // Simplify tram listings - show more akin to trains
    simplify_tram_names(&mut stops, &operator, &state.operators);

    let route = shape.unwrap_or_else(|_| polyline::encode_coordinates(
        stops.iter().filter_map(|s| Some(coord! {x: s.long?, y: s.lat?})), 5).unwrap());

    let mut cancelled = false;
    let find_realtime_trip = find_realtime_trip_with_gtfs(id, &state.vehicles);
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
    }.or_else(|| realtime_from_links(state, &mut stops, &links));

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
                connections: links.unwrap_or_default(),
            }
        ],
        alerts
    };
    
    if let Some((resp, _)) = find_realtime_trip.as_ref() {
        cache_service_data(&state.realtime_cache, resp.clone(), id.as_str(), &data);
    } else if data.branches.first().unwrap().realtime.is_some()
        && let Some(from) = uw!(data.branches.first().unwrap().connections.from.as_ref())
        && let Some((resp, _)) = find_realtime_trip_with_gtfs(from.trip_id.as_str(), &state.vehicles) {
        cache_service_data(&state.realtime_cache, resp.clone(), id.as_str(), &data);
    }
    
    Ok(data)
}

fn realtime_from_links(state: &Arc<GTFSState>, mut stops: &mut Vec<StopsQuery>, links: &Result<Connections, Box<dyn Error>>) -> Option<RealtimeInfo> {
    if let Some(previous_service) = uw!(links.as_ref().ok()?.from.as_ref()) {
        if let Some(previous_service_data) = get_or_cache_all_service_data(state, previous_service.trip_id.as_str()) {
            let branch = previous_service_data.branches.first()?;
            if let Some(realtime) = branch.realtime.as_ref() {
                if let Some(delay) = realtime.delay {
                    let prev_last_stop = branch.stops.last()?;
                    let curr_first_stop = stops.first()?;

                    let mut delay = TimeDelta::milliseconds(delay);

                    // Figure out what date this starts on (today or tomorrow, if on the edge of midnight)
                    let last_stop_time = realtime.date.and_time(NaiveTime::default()).and_utc().add(prev_last_stop.arr);
                    let time_diff =
                        (curr_first_stop.dep.num_milliseconds() % MILLISECONDS_IN_DAY)
                            - (prev_last_stop.arr.num_milliseconds() % MILLISECONDS_IN_DAY);
                    let time_diff = if time_diff < 0 {
                        TimeDelta::minutes(10) + TimeDelta::milliseconds(time_diff % (1000 * 60 * 10))
                    } else {
                        TimeDelta::milliseconds(time_diff)
                    };
                    let date = (last_stop_time + time_diff).with_time(NaiveTime::default()).unwrap();

                    // Apply date to these times for delay calc
                    let scheduled_times = stops.iter().map(|stop| {
                        ScheduledTime {
                            arr: date + stop.arr,
                            dep: date + stop.dep
                        }
                    }).collect_vec();

                    // Subtract delay between the last service ending and this starting
                    delay = delay - (scheduled_times.first().unwrap().dep - last_stop_time);

                    // Usual delay calculation
                    for i in 0..stops.len() {
                        let scheduled_time = &scheduled_times[i];
                        let delayed_time = scheduled_time.arr + delay;
                        stops[i].status = Some(calculate_delay_status(&mut delay, scheduled_time, delayed_time));
                    };

                    let on_previous_journey = realtime.stop > 0;

                    return Some(RealtimeInfo {
                        stop: 0,
                        pct: 0.0,
                        // Show vehicle position if the last vehicle has set off
                        pos: if on_previous_journey {
                            realtime.pos.clone()
                        } else {
                            None
                        },
                        delay: Some(max(delay.num_milliseconds(), 0)),
                        date: date.date_naive(),
                        on_previous: on_previous_journey,
                        vehicle: realtime.vehicle.clone(),
                    });
                }
            }
        }
    }
    None
}

fn calculate_delay_status(delay: &mut TimeDelta, scheduled_time: &ScheduledTime, delayed_time: DateTime<Utc>) -> String {
    let mut status = "".to_string();
    if *delay >= TimeDelta::milliseconds(1000 * 120) || *delay <= TimeDelta::milliseconds(-1000 * 60) {
        status = if scheduled_time.dep.minute() == delayed_time.minute() {
            "On time".to_string()
        } else {
            format!("Exp. {}", delayed_time.format("%H:%M"))
        };

        // Absorb delay in longer layovers
        *delay = *delay - (scheduled_time.dep - scheduled_time.arr);
        if delay.num_milliseconds() < 0 {
            *delay = TimeDelta::default();
            status = "On time".to_string();
        }
    } else {
        *delay = *delay - (scheduled_time.dep - scheduled_time.arr);
        if delay.num_milliseconds() < 0 {
            *delay = TimeDelta::default();
        }
        status = "On time".to_string();
    }
    status
}

pub fn get_alerts(alerts: &GTFSAlerts, trip_id: Option<&String>, route_id: Option<&String>, agency_id: Option<&String>) -> Vec<StopAlert> {
    alerts.iter().flat_map(|a| {
        a.value().iter().filter(|&alert| {
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
            let evaluate_pos = stops[evaluate_index].position();
            let include_last_stop = if evaluate_pos.is_some() && Point::from(evaluate_pos.unwrap()).geodesic_distance(&current_pos_point) <= 50.0 { 1 } else { 0 };

            // Only show Departed 5 mins before departure time
            for i in 0..stops.len() {
                let scheduled_time = &scheduled_times[i];
                let delayed_time = scheduled_time.arr + delay;
                
                if i < current_stop_index - include_last_stop && (scheduled_time.dep - adjust_timestamp(time_now)).num_seconds() < 120 {
                    stops[i].status = Some("Departed".to_string());
                    continue;
                }

                stops[i].status = Some(calculate_delay_status(&mut delay, scheduled_time, delayed_time));

                // Show current delayed stop in major stops list for context (since previous stops don't show delay, can look on time when delayed)
                if stops[current_stop_index].status != Some("On time".to_string()) {
                    stops[current_stop_index].major = true
                }
            };

            Some(RealtimeInfo {
                stop: current_stop_index as i64,
                pct,
                pos: Some(current_pos),
                delay: Some(delay.num_milliseconds()),
                date: scheduled_times.first().unwrap().dep.date_naive(),
                on_previous: false,
                vehicle: get_vehicle_info(trip)
            })
        } else {
            None
        }
    } else {
        Some(RealtimeInfo {
            stop: -1,
            pct: 0.0,
            pos: Some(current_pos),
            delay: None,
            date: time_now.date_naive(),
            on_previous: false,
            vehicle: get_vehicle_info(trip)
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
                return DateTime::from_timestamp(time, 0).map(|d| adjust_timestamp(&d));
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
            stops[i].status = Some(format!("Exp. {}", actual_time.unwrap().format("%H:%M")))
        }
    });
    let actual_times = actual_times.into_iter().filter_map(|x| x).collect_vec();
    assert_eq!(scheduled_times.len(), actual_times.len());

    let adjusted_now = adjust_timestamp(time_now);
    let current = actual_times.iter().find_position(|&t| t >= &adjusted_now).map(|c| c.0);
    let pct = match current {
        None => 1.0,
        Some(0) => 0.0,
        Some(i) => (adjusted_now - actual_times[i - 1]).num_milliseconds() as f64 / (actual_times[i] - actual_times[i - 1]).num_milliseconds() as f64,
    };

    let current = current.unwrap_or(0);
    stops.iter_mut().enumerate().take(current).for_each(|(i, stop)| {
        stop.status = Some(format!("Dep. {}", actual_times[i].format("%H:%M")));
    });

    RealtimeInfo {
        stop: current as i64,
        pct,
        pos: current_pos,
        delay: Some((*actual_times.last().unwrap() - *scheduled_times.last().unwrap()).num_milliseconds()),
        date: date.date_naive(),
        on_previous: false,
        vehicle: get_vehicle_info(trip)
    }
}

fn get_vehicle_info(trip: &FeedEntity) -> VehicleInfo {
    VehicleInfo {
        name: uw!(trip.vehicle.as_ref()?.vehicle.as_ref()).and_then(|vd| vd.label.clone()),
        license: uw!(trip.vehicle.as_ref()?.vehicle.as_ref()).and_then(|vd| vd.license_plate.clone()),
        occupancy_pct: uw!(trip.vehicle.as_ref()?.occupancy_percentage).clone()
    }
}

fn get_start_date(trip: &FeedEntity, time_now: &DateTime<Utc>) -> DateTime<Utc> {
    uw!(trip.vehicle.as_ref()?.trip.as_ref()?.start_date.as_ref()).map(|d| NaiveDate::parse_from_str(d.as_str(), "%Y%m%d").map(|dt| dt.and_time(NaiveTime::default()).and_utc()).ok())
        .flatten().unwrap_or(*time_now)
}

fn simplify_tram_names(stops: &mut Vec<StopsQuery>, operator: &OperatorsQuery, operators: &OperatorColours) {
    match operator.name.as_str() {
        "Edinburgh Trams" | "Tyne & Wear Metro" | "Metrolink" | "SPT Subway" | "London Underground" =>
            stops.iter_mut().for_each(|stop| {
                stop.ind = None;
                stop.display_name = operators.suffixes[&operator.name].replace(stop.name.as_str(), "").to_string();
                if stop.name != "Rail Station" { stop.loc = None; }
            }),
        "West Midlands Metro" | "Nottingham Express Transit (Tram)" | "Docklands Light Railway"
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
    pos: Option<Position>,
    delay: Option<i64>,
    date: NaiveDate,
    on_previous: bool,
    vehicle: VehicleInfo
}

#[derive(Serialize, Clone)]
pub struct VehicleInfo {
    license: Option<String>,
    name: Option<String>,
    occupancy_pct: Option<u32>
}

pub struct ScheduledTime {
    arr: DateTime<Utc>,
    dep: DateTime<Utc>
}

#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
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
    pub route: String,
    pub connections: Connections
}

#[derive(Serialize, Clone)]
pub struct ServiceData {
    pub service: ServiceInfo,
    pub operator: OperatorsQuery,
    pub branches: Vec<ServiceBranch>,
    pub alerts: Vec<StopAlert>
}