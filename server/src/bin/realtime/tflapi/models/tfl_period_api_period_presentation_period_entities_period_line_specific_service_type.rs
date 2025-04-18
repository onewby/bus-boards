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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineSpecificServiceType {
    #[serde(rename = "serviceType", skip_serializing_if = "Option::is_none")]
    pub service_type: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineServiceTypeInfo>>,
    #[serde(rename = "stopServesServiceType", skip_serializing_if = "Option::is_none")]
    pub stop_serves_service_type: Option<bool>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineSpecificServiceType {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineSpecificServiceType {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodLineSpecificServiceType {
            service_type: None,
            stop_serves_service_type: None,
        }
    }
}

