use std::collections::HashMap;
use std::iter;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::{join, time};
use crate::GTFSResponse;

use chrono::{TimeDelta, Utc};
use tokio::sync::Mutex;
use BusBoardsServer::config::{BBConfig, SourceURL};
use crate::db::{DBPool, get_agency, get_route};
use crate::GTFSResponder::DISRUPTIONS;
use crate::lothian::get_lothian_disruptions;
use crate::passenger::get_passenger_disruptions;
use crate::siri::{AffectedStopPoint, create_translated_string, download_siri, get_infolinks_url, Operators, SiriAffectedOperator};
use crate::transit_realtime::{Alert, EntitySelector, TimeRange};
use crate::transit_realtime::alert::{Cause, Effect};


pub async fn disruptions_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    let passenger_alerts_cache = Mutex::new(HashMap::<SourceURL, Vec<Alert>>::new());
    loop {
        // Get provider disruptions
        let (bods_alerts, lothian_alerts, passenger_alerts) = join!(get_bods_disruptions(&db), get_lothian_disruptions(&db), get_passenger_disruptions(&db, &config, &passenger_alerts_cache));

        // Flatten provider disruptions into one Vec
        let mut alerts = Vec::with_capacity(bods_alerts.len() + lothian_alerts.len() + passenger_alerts.len());
        alerts.extend(bods_alerts);
        alerts.extend(lothian_alerts);
        alerts.extend(passenger_alerts);

        // Publish to main feed
        tx.send((DISRUPTIONS, HashMap::new(), alerts)).await.unwrap_or_else(|err| eprintln!("{}", err));

        // Wait until next loop
        time::sleep(time::Duration::from_secs(60*15)).await
    }
}

/// Get BODS disruptions
async fn get_bods_disruptions(db: &Arc<DBPool>) -> Vec<Alert> {
    let siri = download_siri().await;
    let alerts: Vec<Alert> = siri.siri.service_delivery.situation_exchange_delivery.situations.situations.iter().flat_map(|situation| {
        // Map time ranges to GTFS
        let time_ranges: Vec<TimeRange> = situation.validity_period.iter().map(|pw| {
            TimeRange {
                start: Some(pw.start_time.timestamp() as u64),
                end: match pw.end_time.to_owned() {
                    None => Some(Utc::now().add(TimeDelta::weeks(52)).timestamp() as u64),
                    Some(time) => Some(time.timestamp() as u64)
                }
            }
        }).collect();
        // One generic alert for stops
        // Specific advice for each route
        let mut alerts: Vec<Alert> = situation.consequences.consequences.iter().map(|con| {
            // Route/operator-specific alert
            Alert {
                active_period: time_ranges.clone(),
                cause: Some(Cause::OtherCause as i32),
                effect: Some(Effect::OtherEffect as i32),
                description_text: Some(create_translated_string(situation.description.to_string() + (if let Some(advice) = &con.advice { format!(" {}", advice.details) } else { "".to_string() }).as_str())),
                tts_header_text: None,
                tts_description_text: None,
                severity_level: None,
                image: None,
                image_alternative_text: None,
                cause_detail: None,
                effect_detail: None,
                header_text: Some(create_translated_string(situation.summary.to_string())),
                informed_entity: con.affects.networks.affected_network.affected_line.iter().map(|line| {
                    let route = get_route(db, line.affected_operator.operator_ref.to_owned(), line.line_ref.as_ref().unwrap_or(&"".to_string()).to_owned());
                    EntitySelector { agency_id: None, route_id: route.ok(), route_type: None, trip: None, stop_id: None, direction_id: None }
                }).chain(
                    compact_op(&con.affects.operators).map(|op| {
                        let agency = get_agency(db, op.operator_ref.to_owned());
                        EntitySelector { agency_id: agency.ok(), route_id: None, route_type: None, trip: None, stop_id: None, direction_id: None }
                    })
                ).collect(),
                url: get_infolinks_url(&situation.info_links)
            }
        }).filter(|alert: &Alert| !alert.informed_entity.is_empty()).collect();
        let stop_points: Vec<&AffectedStopPoint> = situation.consequences.consequences.iter().flat_map(
            |con| if let Some(sps) = con.affects.stop_points.as_ref() {
                sps.stop_points.iter().collect::<Vec<_>>()
            } else { vec![] }
        ).collect();
        if !stop_points.is_empty() {
            // Generic alert for stops
            alerts.push(Alert {
                active_period: time_ranges.clone(),
                cause: Some(Cause::OtherCause as i32),
                effect: Some(Effect::OtherEffect as i32),
                description_text: Some(create_translated_string(situation.description.clone())),
                header_text: Some(create_translated_string(situation.summary.clone())),
                informed_entity: stop_points.iter().map(|sp| {
                    EntitySelector {
                        agency_id: None,
                        route_id: None,
                        route_type: None,
                        trip: None,
                        stop_id: Some(sp.stop_point_ref.to_string()),
                        direction_id: None
                    }
                }).collect(),
                url: get_infolinks_url(&situation.info_links),
                tts_header_text: None,
                tts_description_text: None,
                severity_level: None,
                image: None,
                image_alternative_text: None,
                cause_detail: None,
                effect_detail: None
            })
        }
        alerts
    }).collect();
    alerts
}

/// Convert Operators enum to an iterator of operators
fn compact_op<'a>(obj: &'a Option<Operators>) -> Box<dyn Iterator<Item = &'a SiriAffectedOperator> + 'a> {
    return match obj {
        None => Box::new(iter::empty()),
        Some(ops) => match ops {
            Operators::AllOperator(_) => Box::new(iter::empty()),
            Operators::SiriAffectedOperatorArray(ops) => Box::new(ops.iter())
        }
    }
}