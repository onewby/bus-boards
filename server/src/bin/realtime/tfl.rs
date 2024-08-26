use std::error::Error;
use std::io::{Cursor, Read};
use std::iter;
use std::str::FromStr;
use std::sync::Arc;
use chrono::{DateTime, TimeDelta, Utc};
use futures::StreamExt;
use itertools::Itertools;
use log::error;
use tokio::sync::mpsc::Sender;
use tokio::time;
use BusBoardsServer::config::BBConfig;
use BusBoardsServer::GTFSResponder::TFL;
use crate::db::{DBPool, get_line_segments, get_route_id};
use crate::{GTFSResponse, tflapi};
use crate::passenger::ActivePeriod;
use crate::siri::create_translated_string;
use crate::tflapi::apis::line_api::line_status_by_ids;
use crate::tflapi::models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruption;
use crate::transit_realtime::{Alert, EntitySelector, TimeRange, TranslatedString};

pub async fn tfl_listener(tx: Sender<GTFSResponse>, config: Arc<BBConfig>, db: Arc<DBPool>) {
    loop {
        let alerts = get_tube_alerts(&db).await
            .inspect_err(|e| error!("{}", e))
            .unwrap_or(vec![]);
        
        // Send to main feed
        tx.send((TFL, vec![], alerts)).await.unwrap_or_else(|err| eprintln!("{}", err));
        
        // Wait until next loop
        time::sleep(time::Duration::from_secs(60)).await
    }
}

pub async fn get_tfl_alerts(db: &Arc<DBPool>) -> Result<Vec<Alert>, Box<dyn Error>> {
    let config = tflapi::apis::configuration::Configuration::new();
    let statuses = line_status_by_ids(&config, vec!["tube,overground,dlr,bus,elizabeth-line,river-bus,tram".to_string()], Some(true)).await?;
    statuses.iter().filter_map(|status| {
        status.line_statuses.as_ref().map(|disruptions| {
            disruptions.iter().map(|disruption| {
                Alert {
                    active_period: disruption.validity_periods.as_ref().map(|vps| {
                        vps.iter().map(|vp| {
                            TimeRange {
                                start: vp.from_date.as_ref().map(|t| DateTime::<Utc>::from_str(t.as_str()).ok().map(|d| d.timestamp() as u64)).flatten(),
                                end: vp.to_date.as_ref().map(|t| DateTime::<Utc>::from_str(t.as_str()).ok().map(|d| d.timestamp() as u64)).flatten(),
                            }
                        }).collect_vec()
                    }).unwrap_or_else(|| {
                        vec![TimeRange::default()]
                    }),
                    informed_entity: iter::once(disruption.line_id.as_ref().map(|route| {
                        EntitySelector {
                            agency_id: None,
                            route_id: get_london_route(route.as_str()).ok(),
                            route_type: None,
                            trip: None,
                            stop_id: None,
                            direction_id: None,
                        }
                    }).unwrap_or_default())
                        .chain(disruption.disruption.as_ref().map(|d| get_disruption_affected_entities(d.as_ref())).into_iter().flatten())
                        .collect_vec(),
                    cause: None,
                    effect: None,
                    url: None,
                    header_text: None,
                    description_text: None,
                    tts_header_text: None,
                    tts_description_text: None,
                    severity_level: None,
                    image: None,
                    image_alternative_text: None,
                    cause_detail: None,
                    effect_detail: None,
                }
            })
        })
    });
    Ok(vec![])
}

pub fn get_disruption_affected_entities(disruption: &TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruption) -> impl Iterator<Item = EntitySelector> + '_ {
    let routes = disruption.affected_routes.as_ref().map(|rs| {
        rs.iter().map(|r| {
            EntitySelector {
                agency_id: None,
                route_id: r.line_id.as_ref().and_then(|l| get_london_route(l.as_str()).ok()),
                route_type: None,
                trip: None,
                stop_id: None,
                direction_id: None,
            }
        })
    });
    let stops = disruption.affected_stops.as_ref().map(|ss| {
        ss.iter().map(|s| {
            EntitySelector {
                agency_id: None,
                route_id: None,
                route_type: None,
                trip: None,
                stop_id: s.naptan_id.clone(),
                direction_id: None,
            }
        })
    });
    
    routes.into_iter().flatten().chain(stops.into_iter().flatten())
}

pub fn get_london_route(route: &str) -> Result<String, Box<dyn Error>> {
    Ok("".to_string())
}

pub async fn get_tube_alerts(db: &Arc<DBPool>) -> Result<Vec<Alert>, Box<dyn Error>> {
    let xml_str = reqwest::get("http://cloud.tfl.gov.uk/TrackerNet/LineStatus").await?.bytes().await?;
    let statuses: ArrayOfLineStatus = yaserde::de::from_reader(Cursor::new(xml_str))?;
    
    Ok(statuses.statuses.iter().filter_map(|status| {
        let route_id = get_route_id(db, "OP5816".to_string(), status.line.name.to_string()).ok()?;
        let stops = get_line_segments(db, route_id.to_string());
        Some(Alert {
            active_period: vec![
                TimeRange {
                    start: Some((Utc::now() - TimeDelta::days(1)).timestamp() as u64),
                    end: Some((Utc::now() + TimeDelta::days(1)).timestamp() as u64),
                }
            ],
            informed_entity: stops.keys().map(|s| {
                EntitySelector {
                    agency_id: None,
                    route_id: None,
                    route_type: None,
                    trip: None,
                    stop_id: Some(s.to_string()),
                    direction_id: None,
                }
            }).chain(iter::once(
                EntitySelector {
                    agency_id: None,
                    route_id: Some(route_id),
                    route_type: None,
                    trip: None,
                    stop_id: None,
                    direction_id: None,
            })).collect(),
            cause: None,
            effect: None,
            url: None,
            header_text: Some(create_translated_string(status.line.name.to_string())),
            description_text: Some(create_translated_string(status.status_details.to_string())),
            tts_header_text: None,
            tts_description_text: None,
            severity_level: None,
            image: None,
            image_alternative_text: None,
            cause_detail: None,
            effect_detail: None,
        })
    }).collect())
}

#[derive(YaDeserialize)]
pub struct ArrayOfLineStatus {
    #[yaserde(rename="LineStatus")]
    pub statuses: Vec<LineStatus>,
}

#[derive(YaDeserialize)]
pub struct LineStatus {
    #[yaserde(rename="ID", attribute)]
    pub id: String,
    #[yaserde(rename="StatusDetails", attribute)]
    pub status_details: String,
    #[yaserde(rename="BranchDisruptions")]
    pub branch_disruptions: BranchDisruptions,
    #[yaserde(rename="Line")]
    pub line: IDName,
    #[yaserde(rename="Status")]
    pub status: Status
}

#[derive(YaDeserialize)]
pub struct BranchDisruptions {
    pub disruptions: Vec<BranchDisruption>
}

#[derive(YaDeserialize)]
pub struct BranchDisruption {
    #[yaserde(rename="StationFrom")]
    pub station_from: IDName,
    #[yaserde(rename="StationTo")]
    pub station_to: IDName,
    #[yaserde(rename="StationVia")]
    pub station_via: Option<IDName>,
    #[yaserde(rename="Status")]
    pub status: Status,
    #[yaserde(rename="Bound")]
    pub bound: Option<Bound>
}

#[derive(YaDeserialize)]
pub struct Bound {
    #[yaserde(rename="Id", attribute)]
    pub id: String,
    #[yaserde(rename="Abbrieviation", attribute)]
    pub abbreviation: String,
    #[yaserde(rename="Name", attribute)]
    pub name: String
}

#[derive(YaDeserialize)]
pub struct IDName {
    #[yaserde(rename="ID", attribute)]
    pub id: String,
    #[yaserde(rename="Name", attribute)]
    pub name: String
}

#[derive(YaDeserialize)]
pub struct Status {
    #[yaserde(rename="ID", attribute)]
    pub id: String,
    #[yaserde(rename="CssClass", attribute)]
    pub css_class: String,
    #[yaserde(rename="Description", attribute)]
    pub description: String,
    #[yaserde(rename="IsActive", attribute)]
    pub is_active: bool,
    pub status_type: StatusType
}

#[derive(YaDeserialize)]
pub struct StatusType {
    #[yaserde(rename="ID", attribute)]
    pub id: String,
    #[yaserde(rename="Description", attribute)]
    pub description: String
}