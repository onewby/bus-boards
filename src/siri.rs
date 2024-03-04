use std::fmt::Display;
use std::io::Read;
use std::str::FromStr;
use chrono::{DateTime, Utc};

use serde::{Serialize, Deserialize};
use zip::ZipArchive;
use crate::transit_realtime::translated_string::Translation;
use crate::transit_realtime::TranslatedString;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SiriSx {
    pub siri: Siri,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Siri {
    pub service_delivery: ServiceDelivery,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceDelivery {
    pub response_timestamp: String,
    pub producer_ref: String,
    pub response_message_identifier: String,
    pub situation_exchange_delivery: SituationExchangeDelivery,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SituationExchangeDelivery {
    pub response_timestamp: String,
    pub situations: Situations,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Situations {
    #[serde(rename = "PtSituationElement")]
    pub situations: Vec<PtSituationElement>
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PtSituationElement {
    pub creation_time: String,
    pub participant_ref: String,
    pub situation_number: String,
    pub source: Source,
    pub progress: String,
    pub validity_period: Vec<PublicationWindow>,
    pub publication_window: PublicationWindow,
    pub miscellaneous_reason: Option<String>,
    pub planned: bool,
    pub summary: String,
    pub description: String,
    pub info_links: Option<InfoLinks>,
    pub consequences: Consequences,
    pub equipment_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Consequences {
    #[serde(rename = "Consequence")]
    pub consequences: Vec<Consequence>
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Advice {
    pub details: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AllOperators {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Operators {
    AllOperator(AllOperators),
    SiriAffectedOperatorArray(Vec<SiriAffectedOperator>)
}

impl Default for Operators {
    fn default() -> Self {
        Operators::AllOperator(AllOperators::default())
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SiriAffectedOperator {
    pub operator_ref: String,
    pub operator_name: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedStopPoint {
    pub stop_point_ref: String,
    pub stop_point_name: String,
    pub location: Location,
    pub affected_modes: AffectedModes,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedModes {
    pub mode: Mode,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Mode {
    pub vehicle_mode: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Blocking {
    pub journey_planner: bool,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Consequence {
    pub condition: String,
    pub severity: String,
    pub affects: Affects,
    pub advice: Option<Advice>,
    pub blocking: Option<Blocking>,
    pub delays: Option<Delays>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Affects {
    pub networks: Networks,
    pub stop_points: Option<StopPoints>,
    pub operators: Option<Operators>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StopPoints {
    #[serde(rename = "AffectedStopPoint")]
    pub stop_points: Vec<AffectedStopPoint>
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Networks {
    pub affected_network: AffectedNetwork,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedNetwork {
    pub vehicle_mode: String,
    #[serde(default)]
    pub affected_line: Vec<AffectedLine>,
    pub all_lines: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Direction {
    pub direction_ref: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AffectedLine {
    pub affected_operator: SiriAffectedOperator,
    pub line_ref: Option<String>,
    pub direction: Option<Direction>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum AffectedOperatorUnion {
    AffectedOperatorClass(SiriAffectedOperator),
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

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Delays {
    pub delay: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InfoLinks {
    pub info_link: Vec<InfoLink>
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InfoLink {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PublicationWindow {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    pub source_type: String,
    pub time_of_communication: String,
}

pub async fn download_siri() -> SiriSx {
    if let Ok(result) = reqwest::get("https://data.bus-data.dft.gov.uk/disruptions/download/bulk_archive").await {
        if let Ok(bytes) = result.bytes().await {
            if let Ok(mut archive) = ZipArchive::new(std::io::Cursor::new(bytes)) {
                if let Ok(zip_file) = archive.by_name("sirisx.xml") {
                    let siri_result = serde_xml_rust::from_reader(std::io::Cursor::new(zip_file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>()));
                    if let Ok(siri) = siri_result {
                        return SiriSx { siri }
                    } else {
                        eprintln!("{}", siri_result.unwrap_err())
                    }
                }
            }
        }
    }
    SiriSx::default()
}

pub fn create_translated_string(str: String) -> TranslatedString {
    TranslatedString {
        translation: vec![
            Translation {
                text: str,
                language: Some("en".to_string())
            }
        ],
    }
}

pub fn get_infolinks_url(il_option: &Option<InfoLinks>) -> Option<TranslatedString> {
    return if let Some(il) = il_option {
        il.info_link.first().map(|link| TranslatedString {
            translation: vec![
                Translation {
                    text: link.uri.to_string(),
                    language: Some("en".to_string())
                }
            ],
        })
    } else {
        None
    }
}
