use std::fmt::Display;
use std::iter;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use bytes::Buf;
use tokio::sync::mpsc::Sender;
use tokio::time;
use crate::GTFSResponse;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeDelta, Utc};
use crate::db::{DBPool, get_agency, get_route};
use crate::disruptions::AffectedStopPointUnion::AffectedStopPointElement;
use crate::disruptions::AffectedLineUnion::AffectedLineElement;
use crate::disruptions::Operators::AllOperator;
use crate::GTFSResponder::DISRUPTIONS;
use crate::transit_realtime::{Alert, EntitySelector, TimeRange, TranslatedString};
use crate::transit_realtime::alert::{Cause, Effect};
use crate::transit_realtime::translated_string::Translation;

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SiriSx {
    pub siri: Siri,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Siri {
    pub service_delivery: ServiceDelivery,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceDelivery {
    pub response_timestamp: String,
    pub producer_ref: String,
    pub response_message_identifier: String,
    pub situation_exchange_delivery: SituationExchangeDelivery,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SituationExchangeDelivery {
    pub response_timestamp: String,
    pub situations: Situations,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Situations {
    pub pt_situation_element: Vec<PtSituationElement>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PtSituationElement {
    pub creation_time: String,
    pub participant_ref: String,
    pub situation_number: String,
    pub source: Source,
    pub progress: String,
    pub validity_period: ValidityPeriod,
    pub publication_window: PublicationWindow,
    pub miscellaneous_reason: Option<String>,
    pub planned: bool,
    pub summary: String,
    pub description: String,
    pub info_links: Option<InfoLinks>,
    pub consequences: Consequences,
    pub equipment_reason: Option<String>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Consequences {
    pub consequence: ConsequenceUnion,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum ConsequenceUnion {
    ConsequenceElementArray(Vec<ConsequenceElement>),
    Consequence(ConsequenceElement),
}

impl Default for ConsequenceUnion {
    fn default() -> Self {
        ConsequenceUnion::Consequence(ConsequenceElement::default())
    }
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Advice {
    pub details: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedOperatorClass {
    pub operator_ref: String,
    pub operator_name: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum Ref {
    Integer(i64),
    String(String),
}

impl Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Ref::Integer(r) => r.to_string(),
            Ref::String(r) => r.to_string()
        };
        write!(f, "{}", str)
    }
}

impl Default for Ref {
    fn default() -> Self {
        Ref::Integer(0)
    }
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AllOperators {
    pub all_operators: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum Operators {
    AllOperator(AllOperators),
    SiriAffectedOperatorArray(Vec<SiriAffectedOperator>),
    SiriAffectedOperator(SiriAffectedOperator)
}

impl Default for Operators {
    fn default() -> Self {
        AllOperator(AllOperators::default())
    }
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SiriAffectedOperator {
    pub operator_ref: String,
    pub operator_name: String
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StopPoints {
    pub affected_stop_point: AffectedStopPointUnion,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum AffectedStopPointUnion {
    AffectedStopPointElementArray(Vec<AffectedStopPoint>),
    AffectedStopPointElement(AffectedStopPoint),
}

impl Default for AffectedStopPointUnion {
    fn default() -> Self {
        AffectedStopPointElement(AffectedStopPoint::default())
    }
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedStopPoint {
    pub stop_point_ref: Ref,
    pub stop_point_name: String,
    pub location: Location,
    pub affected_modes: AffectedModes,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedModes {
    pub mode: Mode,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Mode {
    pub vehicle_mode: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Blocking {
    pub journey_planner: bool,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ConsequenceElement {
    pub condition: String,
    pub severity: String,
    pub affects: Affects,
    pub advice: Option<Advice>,
    pub blocking: Option<Blocking>,
    pub delays: Option<Delays>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Affects {
    pub networks: Networks,
    pub stop_points: Option<StopPoints>,
    pub operators: Option<Operators>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Networks {
    pub affected_network: AffectedNetwork,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedNetwork {
    pub vehicle_mode: String,
    pub affected_line: Option<AffectedLineUnion>,
    pub all_lines: Option<String>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum AffectedLineUnion {
    AffectedLineElementArray(Vec<AffectedLine>),
    AffectedLineElement(AffectedLine),
}

impl Default for AffectedLineUnion {
    fn default() -> Self {
        AffectedLineElement(AffectedLine::default())
    }
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Direction {
    pub direction_ref: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedLine {
    pub affected_operator: AffectedOperatorUnion,
    pub line_ref: Option<Ref>,
    pub direction: Option<Direction>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum AffectedOperatorUnion {
    AffectedOperatorClass(AffectedOperatorClass),
    String(String),
}

impl Default for AffectedOperatorUnion {
    fn default() -> Self {
        AffectedOperatorUnion::String("".to_string())
    }
}

impl AffectedOperatorUnion {
    fn operator_ref(&self) -> String {
        match self {
            AffectedOperatorUnion::AffectedOperatorClass(aff_op) => aff_op.operator_ref.clone(),
            AffectedOperatorUnion::String(str) => str.clone()
        }
    }
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Delays {
    pub delay: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InfoLinks {
    pub info_link: InfoLink,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InfoLink {
    pub uri: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PublicationWindow {
    pub start_time: String,
    pub end_time: Option<String>,
}

#[derive(Serialize, Deserialize, YaDeserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    pub source_type: String,
    pub time_of_communication: String,
}

#[derive(Serialize, Deserialize, YaDeserialize, Debug)]
#[serde(untagged)]
pub enum ValidityPeriod {
    PublicationWindow(PublicationWindow),
    PublicationWindowArray(Vec<PublicationWindow>),
}

impl Default for ValidityPeriod {
    fn default() -> Self {
        ValidityPeriod::PublicationWindow(PublicationWindow::default())
    }
}


pub async fn disruptions_listener(tx: Sender<GTFSResponse>, db: Arc<DBPool>) -> () {
    loop {
        let siri = download_siri().await;
        let alerts: Vec<Alert> = siri.siri.service_delivery.situation_exchange_delivery.situations.pt_situation_element.iter().flat_map(|situation| {
            let time_ranges: Vec<TimeRange> = compact_vp(&situation.validity_period).map(|pw| {
                TimeRange {
                    start: Some(DateTime::<Utc>::from_str(pw.start_time.as_str()).unwrap_or(Utc::now()).timestamp() as u64),
                    end: match pw.end_time.to_owned() {
                        None => Some(Utc::now().add(TimeDelta::weeks(52)).timestamp() as u64),
                        Some(time) => Some(DateTime::<Utc>::from_str(time.as_str()).unwrap_or(Utc::now().add(TimeDelta::weeks(52))).timestamp() as u64)
                    }
                }
            }).collect();
            // One generic for stops
            // Specific advice for each route
            let mut alerts: Vec<Alert> = compact_con(&situation.consequences.consequence).map(|con| {
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
                    informed_entity: compact_al(&con.affects.networks.affected_network.affected_line).map(|line| {
                        let route = get_route(&db, line.affected_operator.operator_ref().as_str(), line.line_ref.as_ref().unwrap_or(&Ref::Integer(0)).to_string().as_str());
                        EntitySelector { agency_id: None, route_id: route.ok(), route_type: None, trip: None, stop_id: None, direction_id: None }
                    }).chain(
                        compact_op(&con.affects.operators).map(|op| {
                            let agency = get_agency(&db, op.operator_ref.as_str());
                            EntitySelector { agency_id: agency.ok(), route_id: None, route_type: None, trip: None, stop_id: None, direction_id: None }
                        })
                    ).collect(),
                    url: Some(create_translated_string(situation.info_links.as_ref().unwrap_or(&InfoLinks::default()).info_link.uri.to_string()))
            }
            }).filter(|alert: &Alert| alert.informed_entity.len() > 0).collect();
            let stop_points: Vec<&AffectedStopPoint> = compact_con(&situation.consequences.consequence).flat_map(|con| compact_sp(&con.affects.stop_points).collect::<Vec<_>>()).collect();
            if stop_points.len() > 0 {
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
                    url: Some(create_translated_string(situation.info_links.as_ref().unwrap_or(&InfoLinks::default()).info_link.uri.to_string())),
                    tts_header_text: None,
                    tts_description_text: None,
                    severity_level: None,
                    image: None,
                    image_alternative_text: None,
                    cause_detail: None,
                    effect_detail: None
                })
            }
            return alerts
        }).collect();

        tx.send((DISRUPTIONS, vec![], alerts)).await.unwrap_or_else(|err| eprintln!("{}", err));

        time::sleep(time::Duration::from_secs(60)).await
    }
}

async fn download_siri() -> SiriSx {
    if let Ok(result) = reqwest::get("https://data.bus-data.dft.gov.uk/api/v1/siri-sx/").await {
        if let Ok(bytes) = result.bytes().await {
            let siri_result: Result<SiriSx, String> = yaserde::de::from_reader(std::io::Cursor::new(bytes));
            if let Ok(siri) = siri_result {
                return siri
            } else {
                eprintln!("{}", siri_result.unwrap_err())
            }
        }
    }
    SiriSx::default()
}

fn compact_vp<'a>(obj: &'a ValidityPeriod) -> Box<dyn Iterator<Item = &'a PublicationWindow> + 'a> {
    return match obj {
        ValidityPeriod::PublicationWindow(vp) => {
            Box::new(iter::once(vp))
        }
        ValidityPeriod::PublicationWindowArray(vps) => {
            Box::new(vps.iter())
        }
    }
}

fn compact_con<'a>(obj: &'a ConsequenceUnion) -> Box<dyn Iterator<Item = &'a ConsequenceElement> + 'a> {
    return match obj {
        ConsequenceUnion::ConsequenceElementArray(cons) => Box::new(cons.iter()),
        ConsequenceUnion::Consequence(con) => Box::new(iter::once(con))
    }
}

fn compact_al<'a>(obj: &'a Option<AffectedLineUnion>) -> Box<dyn Iterator<Item = &'a AffectedLine> + 'a> {
    return match obj {
        None => Box::new(iter::empty()),
        Some(lines) => match lines {
            AffectedLineUnion::AffectedLineElement(line) => Box::new(iter::once(line)),
            AffectedLineUnion::AffectedLineElementArray(lines) => Box::new(lines.iter())
        }
    }
}

fn compact_op<'a>(obj: &'a Option<Operators>) -> Box<dyn Iterator<Item = &'a SiriAffectedOperator> + 'a> {
    return match obj {
        None => Box::new(iter::empty()),
        Some(ops) => match ops {
            Operators::AllOperator(_) => Box::new(iter::empty()),
            Operators::SiriAffectedOperatorArray(ops) => Box::new(ops.iter()),
            Operators::SiriAffectedOperator(op) => Box::new(iter::once(op))
        }
    }
}

fn compact_sp<'a>(obj: &'a Option<StopPoints>) -> Box<dyn Iterator<Item = &'a AffectedStopPoint> + 'a> {
    return match obj {
        None => Box::new(iter::empty()),
        Some(stops) => match &stops.affected_stop_point {
            AffectedStopPointUnion::AffectedStopPointElementArray(points) => Box::new(points.iter()),
            AffectedStopPointUnion::AffectedStopPointElement(point) => Box::new(iter::once(point))
        }
    }
}

fn create_translated_string(str: String) -> TranslatedString {
    TranslatedString {
        translation: vec![
            Translation {
                text: str,
                language: Some("en".to_string())
            }
        ],
    }
}
