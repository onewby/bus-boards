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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstruction {
    #[serde(rename = "summary", skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(rename = "detailed", skip_serializing_if = "Option::is_none")]
    pub detailed: Option<String>,
    #[serde(rename = "steps", skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstructionStep>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstruction {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstruction {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodInstruction {
            summary: None,
            detailed: None,
            steps: None,
        }
    }
}

