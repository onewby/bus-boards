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
pub struct TflPeriodApiPeriodCommonPeriodJourneyPlannerPeriodJpElevation {
    #[serde(rename = "distance", skip_serializing_if = "Option::is_none")]
    pub distance: Option<i32>,
    #[serde(rename = "startLat", skip_serializing_if = "Option::is_none")]
    pub start_lat: Option<f64>,
    #[serde(rename = "startLon", skip_serializing_if = "Option::is_none")]
    pub start_lon: Option<f64>,
    #[serde(rename = "endLat", skip_serializing_if = "Option::is_none")]
    pub end_lat: Option<f64>,
    #[serde(rename = "endLon", skip_serializing_if = "Option::is_none")]
    pub end_lon: Option<f64>,
    #[serde(rename = "heightFromPreviousPoint", skip_serializing_if = "Option::is_none")]
    pub height_from_previous_point: Option<i32>,
    #[serde(rename = "gradient", skip_serializing_if = "Option::is_none")]
    pub gradient: Option<f64>,
}

impl TflPeriodApiPeriodCommonPeriodJourneyPlannerPeriodJpElevation {
    pub fn new() -> TflPeriodApiPeriodCommonPeriodJourneyPlannerPeriodJpElevation {
        TflPeriodApiPeriodCommonPeriodJourneyPlannerPeriodJpElevation {
            distance: None,
            start_lat: None,
            start_lon: None,
            end_lat: None,
            end_lon: None,
            height_from_previous_point: None,
            gradient: None,
        }
    }
}
