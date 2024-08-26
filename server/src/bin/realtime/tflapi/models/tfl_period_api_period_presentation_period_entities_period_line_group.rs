/*
 * Transport for London Unified API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: v1
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::tflapi::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineGroup {
    #[serde(rename = "naptanIdReference", skip_serializing_if = "Option::is_none")]
    pub naptan_id_reference: Option<String>,
    #[serde(rename = "stationAtcoCode", skip_serializing_if = "Option::is_none")]
    pub station_atco_code: Option<String>,
    #[serde(rename = "lineIdentifier", skip_serializing_if = "Option::is_none")]
    pub line_identifier: Option<Vec<String>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineGroup {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineGroup {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineGroup {
            naptan_id_reference: None,
            station_atco_code: None,
            line_identifier: None,
        }
    }
}

