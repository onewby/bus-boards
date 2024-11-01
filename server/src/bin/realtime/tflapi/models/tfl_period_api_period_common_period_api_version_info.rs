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
pub struct TflPeriodApiPeriodCommonPeriodApiVersionInfo {
    #[serde(rename = "label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "assemblies", skip_serializing_if = "Option::is_none")]
    pub assemblies: Option<Vec<String>>,
}

impl TflPeriodApiPeriodCommonPeriodApiVersionInfo {
    pub fn new() -> TflPeriodApiPeriodCommonPeriodApiVersionInfo {
        TflPeriodApiPeriodCommonPeriodApiVersionInfo {
            label: None,
            timestamp: None,
            version: None,
            assemblies: None,
        }
    }
}

