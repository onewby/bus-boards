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

/// TflPeriodApiPeriodPresentationPeriodEntitiesPeriodNetworkStatus : Represent travel network status
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodNetworkStatus {
    #[serde(rename = "operator", skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(rename = "statusLevel", skip_serializing_if = "Option::is_none")]
    pub status_level: Option<i32>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodNetworkStatus {
    /// Represent travel network status
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodNetworkStatus {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodNetworkStatus {
            operator: None,
            status: None,
            message: None,
            status_level: None,
        }
    }
}
