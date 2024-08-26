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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodRecommendationResponse {
    #[serde(rename = "recommendations", skip_serializing_if = "Option::is_none")]
    pub recommendations: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodRecommendation>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodRecommendationResponse {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodRecommendationResponse {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodRecommendationResponse {
            recommendations: None,
        }
    }
}
