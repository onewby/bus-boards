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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPredictionTiming {
    #[serde(rename = "countdownServerAdjustment", skip_serializing_if = "Option::is_none")]
    pub countdown_server_adjustment: Option<String>,
    #[serde(rename = "source", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(rename = "insert", skip_serializing_if = "Option::is_none")]
    pub insert: Option<String>,
    #[serde(rename = "read", skip_serializing_if = "Option::is_none")]
    pub read: Option<String>,
    #[serde(rename = "sent", skip_serializing_if = "Option::is_none")]
    pub sent: Option<String>,
    #[serde(rename = "received", skip_serializing_if = "Option::is_none")]
    pub received: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPredictionTiming {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPredictionTiming {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPredictionTiming {
            countdown_server_adjustment: None,
            source: None,
            insert: None,
            read: None,
            sent: None,
            received: None,
        }
    }
}
