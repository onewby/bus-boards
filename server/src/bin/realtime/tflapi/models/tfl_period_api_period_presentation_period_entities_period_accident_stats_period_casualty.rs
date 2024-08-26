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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodAccidentStatsPeriodCasualty {
    #[serde(rename = "age", skip_serializing_if = "Option::is_none")]
    pub age: Option<i32>,
    #[serde(rename = "class", skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(rename = "severity", skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(rename = "mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(rename = "ageBand", skip_serializing_if = "Option::is_none")]
    pub age_band: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodAccidentStatsPeriodCasualty {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodAccidentStatsPeriodCasualty {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodAccidentStatsPeriodCasualty {
            age: None,
            class: None,
            severity: None,
            mode: None,
            age_band: None,
        }
    }
}
