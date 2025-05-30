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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareCaveat {
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareCaveat {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareCaveat {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodJourneyPlannerPeriodFareCaveat {
            text: None,
            r#type: None,
        }
    }
}

