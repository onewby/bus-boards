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
pub struct TflPeriodApiPeriodCommonPeriodPostcodeInput {
    #[serde(rename = "postcode", skip_serializing_if = "Option::is_none")]
    pub postcode: Option<String>,
}

impl TflPeriodApiPeriodCommonPeriodPostcodeInput {
    pub fn new() -> TflPeriodApiPeriodCommonPeriodPostcodeInput {
        TflPeriodApiPeriodCommonPeriodPostcodeInput {
            postcode: None,
        }
    }
}

