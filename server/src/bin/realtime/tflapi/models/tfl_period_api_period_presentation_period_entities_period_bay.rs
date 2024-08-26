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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBay {
    #[serde(rename = "bayType", skip_serializing_if = "Option::is_none")]
    pub bay_type: Option<String>,
    #[serde(rename = "bayCount", skip_serializing_if = "Option::is_none")]
    pub bay_count: Option<i32>,
    #[serde(rename = "free", skip_serializing_if = "Option::is_none")]
    pub free: Option<i32>,
    #[serde(rename = "occupied", skip_serializing_if = "Option::is_none")]
    pub occupied: Option<i32>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBay {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBay {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBay {
            bay_type: None,
            bay_count: None,
            free: None,
            occupied: None,
        }
    }
}

