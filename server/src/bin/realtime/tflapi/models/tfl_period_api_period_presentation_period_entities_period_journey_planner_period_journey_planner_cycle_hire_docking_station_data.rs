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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodJourneyPlannerCycleHireDockingStationData {
    #[serde(rename = "originNumberOfBikes", skip_serializing_if = "Option::is_none")]
    pub origin_number_of_bikes: Option<i32>,
    #[serde(rename = "destinationNumberOfBikes", skip_serializing_if = "Option::is_none")]
    pub destination_number_of_bikes: Option<i32>,
    #[serde(rename = "originNumberOfEmptySlots", skip_serializing_if = "Option::is_none")]
    pub origin_number_of_empty_slots: Option<i32>,
    #[serde(rename = "destinationNumberOfEmptySlots", skip_serializing_if = "Option::is_none")]
    pub destination_number_of_empty_slots: Option<i32>,
    #[serde(rename = "originId", skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,
    #[serde(rename = "destinationId", skip_serializing_if = "Option::is_none")]
    pub destination_id: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodJourneyPlannerCycleHireDockingStationData {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodJourneyPlannerCycleHireDockingStationData {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodJourneyPlannerCycleHireDockingStationData {
            origin_number_of_bikes: None,
            destination_number_of_bikes: None,
            origin_number_of_empty_slots: None,
            destination_number_of_empty_slots: None,
            origin_id: None,
            destination_id: None,
        }
    }
}

