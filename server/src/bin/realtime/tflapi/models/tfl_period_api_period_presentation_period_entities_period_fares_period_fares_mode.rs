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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFaresMode {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFaresMode {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFaresMode {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFaresMode {
            id: None,
            name: None,
            description: None,
        }
    }
}
