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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodOrderedRoute {
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "naptanIds", skip_serializing_if = "Option::is_none")]
    pub naptan_ids: Option<Vec<String>>,
    #[serde(rename = "serviceType", skip_serializing_if = "Option::is_none")]
    pub service_type: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodOrderedRoute {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodOrderedRoute {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodOrderedRoute {
            name: None,
            naptan_ids: None,
            service_type: None,
        }
    }
}

