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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSectionNaptanEntrySequence {
    #[serde(rename = "ordinal", skip_serializing_if = "Option::is_none")]
    pub ordinal: Option<i32>,
    #[serde(rename = "stopPoint", skip_serializing_if = "Option::is_none")]
    pub stop_point: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStopPoint>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSectionNaptanEntrySequence {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSectionNaptanEntrySequence {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSectionNaptanEntrySequence {
            ordinal: None,
            stop_point: None,
        }
    }
}

