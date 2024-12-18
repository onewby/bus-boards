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
pub struct TflPeriodApiPeriodCommonPeriodPlaceGeo {
    #[serde(rename = "swLat", skip_serializing_if = "Option::is_none")]
    pub sw_lat: Option<f64>,
    #[serde(rename = "swLon", skip_serializing_if = "Option::is_none")]
    pub sw_lon: Option<f64>,
    #[serde(rename = "neLat", skip_serializing_if = "Option::is_none")]
    pub ne_lat: Option<f64>,
    #[serde(rename = "neLon", skip_serializing_if = "Option::is_none")]
    pub ne_lon: Option<f64>,
    #[serde(rename = "lat", skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(rename = "lon", skip_serializing_if = "Option::is_none")]
    pub lon: Option<f64>,
}

impl TflPeriodApiPeriodCommonPeriodPlaceGeo {
    pub fn new() -> TflPeriodApiPeriodCommonPeriodPlaceGeo {
        TflPeriodApiPeriodCommonPeriodPlaceGeo {
            sw_lat: None,
            sw_lon: None,
            ne_lat: None,
            ne_lon: None,
            lat: None,
            lon: None,
        }
    }
}

