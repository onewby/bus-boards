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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineModeGroup {
    #[serde(rename = "modeName", skip_serializing_if = "Option::is_none")]
    pub mode_name: Option<String>,
    #[serde(rename = "lineIdentifier", skip_serializing_if = "Option::is_none")]
    pub line_identifier: Option<Vec<String>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineModeGroup {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineModeGroup {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineModeGroup {
            mode_name: None,
            line_identifier: None,
        }
    }
}
