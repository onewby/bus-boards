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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSearchResponse {
    #[serde(rename = "input", skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(rename = "searchMatches", skip_serializing_if = "Option::is_none")]
    pub search_matches: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSearchMatch>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSearchResponse {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSearchResponse {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSearchResponse {
            input: None,
            search_matches: None,
        }
    }
}
