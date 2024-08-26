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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodTimetableResponse {
    #[serde(rename = "lineId", skip_serializing_if = "Option::is_none")]
    pub line_id: Option<String>,
    #[serde(rename = "lineName", skip_serializing_if = "Option::is_none")]
    pub line_name: Option<String>,
    #[serde(rename = "direction", skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(rename = "pdfUrl", skip_serializing_if = "Option::is_none")]
    pub pdf_url: Option<String>,
    #[serde(rename = "stations", skip_serializing_if = "Option::is_none")]
    pub stations: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodMatchedStop>>,
    #[serde(rename = "stops", skip_serializing_if = "Option::is_none")]
    pub stops: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodMatchedStop>>,
    #[serde(rename = "timetable", skip_serializing_if = "Option::is_none")]
    pub timetable: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodTimetable>>,
    #[serde(rename = "disambiguation", skip_serializing_if = "Option::is_none")]
    pub disambiguation: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodTimetablesPeriodDisambiguation>>,
    #[serde(rename = "statusErrorMessage", skip_serializing_if = "Option::is_none")]
    pub status_error_message: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodTimetableResponse {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodTimetableResponse {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodTimetableResponse {
            line_id: None,
            line_name: None,
            direction: None,
            pdf_url: None,
            stations: None,
            stops: None,
            timetable: None,
            disambiguation: None,
            status_error_message: None,
        }
    }
}
