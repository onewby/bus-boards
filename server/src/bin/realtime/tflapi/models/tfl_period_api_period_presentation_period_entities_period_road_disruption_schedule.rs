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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionSchedule {
    #[serde(rename = "startTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionSchedule {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionSchedule {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionSchedule {
            start_time: None,
            end_time: None,
        }
    }
}

