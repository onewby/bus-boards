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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineServiceType {
    #[serde(rename = "lineName", skip_serializing_if = "Option::is_none")]
    pub line_name: Option<String>,
    #[serde(rename = "lineSpecificServiceTypes", skip_serializing_if = "Option::is_none")]
    pub line_specific_service_types: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineSpecificServiceType>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineServiceType {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineServiceType {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineServiceType {
            line_name: None,
            line_specific_service_types: None,
        }
    }
}
