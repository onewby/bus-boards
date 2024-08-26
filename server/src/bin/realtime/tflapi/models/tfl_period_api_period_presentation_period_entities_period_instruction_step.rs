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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstructionStep {
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "turnDirection", skip_serializing_if = "Option::is_none")]
    pub turn_direction: Option<String>,
    #[serde(rename = "streetName", skip_serializing_if = "Option::is_none")]
    pub street_name: Option<String>,
    #[serde(rename = "distance", skip_serializing_if = "Option::is_none")]
    pub distance: Option<i32>,
    #[serde(rename = "cumulativeDistance", skip_serializing_if = "Option::is_none")]
    pub cumulative_distance: Option<i32>,
    #[serde(rename = "skyDirection", skip_serializing_if = "Option::is_none")]
    pub sky_direction: Option<i32>,
    #[serde(rename = "skyDirectionDescription", skip_serializing_if = "Option::is_none")]
    pub sky_direction_description: Option<SkyDirectionDescription>,
    #[serde(rename = "cumulativeTravelTime", skip_serializing_if = "Option::is_none")]
    pub cumulative_travel_time: Option<i32>,
    #[serde(rename = "latitude", skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(rename = "longitude", skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(rename = "pathAttribute", skip_serializing_if = "Option::is_none")]
    pub path_attribute: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPathAttribute>>,
    #[serde(rename = "descriptionHeading", skip_serializing_if = "Option::is_none")]
    pub description_heading: Option<String>,
    #[serde(rename = "trackType", skip_serializing_if = "Option::is_none")]
    pub track_type: Option<TrackType>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstructionStep {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstructionStep {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstructionStep {
            description: None,
            turn_direction: None,
            street_name: None,
            distance: None,
            cumulative_distance: None,
            sky_direction: None,
            sky_direction_description: None,
            cumulative_travel_time: None,
            latitude: None,
            longitude: None,
            path_attribute: None,
            description_heading: None,
            track_type: None,
        }
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum SkyDirectionDescription {
    #[serde(rename = "North")]
    North,
    #[serde(rename = "NorthEast")]
    NorthEast,
    #[serde(rename = "East")]
    East,
    #[serde(rename = "SouthEast")]
    SouthEast,
    #[serde(rename = "South")]
    South,
    #[serde(rename = "SouthWest")]
    SouthWest,
    #[serde(rename = "West")]
    West,
    #[serde(rename = "NorthWest")]
    NorthWest,
}

impl Default for SkyDirectionDescription {
    fn default() -> SkyDirectionDescription {
        Self::North
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TrackType {
    #[serde(rename = "CycleSuperHighway")]
    CycleSuperHighway,
    #[serde(rename = "CanalTowpath")]
    CanalTowpath,
    #[serde(rename = "QuietRoad")]
    QuietRoad,
    #[serde(rename = "ProvisionForCyclists")]
    ProvisionForCyclists,
    #[serde(rename = "BusyRoads")]
    BusyRoads,
    #[serde(rename = "None")]
    None,
    #[serde(rename = "PushBike")]
    PushBike,
    #[serde(rename = "Quietway")]
    Quietway,
}

impl Default for TrackType {
    fn default() -> TrackType {
        Self::CycleSuperHighway
    }
}
