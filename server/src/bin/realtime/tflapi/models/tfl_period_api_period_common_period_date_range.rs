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
pub struct TflPeriodApiPeriodCommonPeriodDateRange {
    #[serde(rename = "startDate", skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate", skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

impl TflPeriodApiPeriodCommonPeriodDateRange {
    pub fn new() -> TflPeriodApiPeriodCommonPeriodDateRange {
        TflPeriodApiPeriodCommonPeriodDateRange {
            start_date: None,
            end_date: None,
        }
    }
}

