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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustments {
    #[serde(rename = "earliest", skip_serializing_if = "Option::is_none")]
    pub earliest: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustment>>,
    #[serde(rename = "earlier", skip_serializing_if = "Option::is_none")]
    pub earlier: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustment>>,
    #[serde(rename = "later", skip_serializing_if = "Option::is_none")]
    pub later: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustment>>,
    #[serde(rename = "latest", skip_serializing_if = "Option::is_none")]
    pub latest: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustment>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustments {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustments {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodTimeAdjustments {
            earliest: None,
            earlier: None,
            later: None,
            latest: None,
        }
    }
}

