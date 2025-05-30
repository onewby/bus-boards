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

/// TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBikePointOccupancy : Bike point occupancy
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBikePointOccupancy {
    /// Id of the bike point such as BikePoints_1
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Name / Common name of the bike point
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Total bike counts
    #[serde(rename = "bikesCount", skip_serializing_if = "Option::is_none")]
    pub bikes_count: Option<i32>,
    /// Empty docks
    #[serde(rename = "emptyDocks", skip_serializing_if = "Option::is_none")]
    pub empty_docks: Option<i32>,
    /// Total docks available
    #[serde(rename = "totalDocks", skip_serializing_if = "Option::is_none")]
    pub total_docks: Option<i32>,
    /// Total standard bikes count
    #[serde(rename = "standardBikesCount", skip_serializing_if = "Option::is_none")]
    pub standard_bikes_count: Option<i32>,
    /// Total ebikes count
    #[serde(rename = "eBikesCount", skip_serializing_if = "Option::is_none")]
    pub e_bikes_count: Option<i32>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBikePointOccupancy {
    /// Bike point occupancy
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBikePointOccupancy {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodBikePointOccupancy {
            id: None,
            name: None,
            bikes_count: None,
            empty_docks: None,
            total_docks: None,
            standard_bikes_count: None,
            e_bikes_count: None,
        }
    }
}

