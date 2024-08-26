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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodActiveServiceType {
    #[serde(rename = "mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(rename = "serviceType", skip_serializing_if = "Option::is_none")]
    pub service_type: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodActiveServiceType {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodActiveServiceType {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodActiveServiceType {
            mode: None,
            service_type: None,
        }
    }
}
