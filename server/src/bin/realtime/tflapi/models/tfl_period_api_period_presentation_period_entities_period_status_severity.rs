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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStatusSeverity {
    #[serde(rename = "modeName", skip_serializing_if = "Option::is_none")]
    pub mode_name: Option<String>,
    #[serde(rename = "severityLevel", skip_serializing_if = "Option::is_none")]
    pub severity_level: Option<i32>,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStatusSeverity {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStatusSeverity {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStatusSeverity {
            mode_name: None,
            severity_level: None,
            description: None,
        }
    }
}
