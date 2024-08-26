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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareTap {
    #[serde(rename = "atcoCode", skip_serializing_if = "Option::is_none")]
    pub atco_code: Option<String>,
    #[serde(rename = "tapDetails", skip_serializing_if = "Option::is_none")]
    pub tap_details: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareTapDetails>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareTap {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareTap {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareTap {
            atco_code: None,
            tap_details: None,
        }
    }
}
