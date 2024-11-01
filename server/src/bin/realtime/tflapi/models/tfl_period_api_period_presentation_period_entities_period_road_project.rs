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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadProject {
    #[serde(rename = "projectId", skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(rename = "schemeName", skip_serializing_if = "Option::is_none")]
    pub scheme_name: Option<String>,
    #[serde(rename = "projectName", skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    #[serde(rename = "projectDescription", skip_serializing_if = "Option::is_none")]
    pub project_description: Option<String>,
    #[serde(rename = "projectPageUrl", skip_serializing_if = "Option::is_none")]
    pub project_page_url: Option<String>,
    #[serde(rename = "consultationPageUrl", skip_serializing_if = "Option::is_none")]
    pub consultation_page_url: Option<String>,
    #[serde(rename = "consultationStartDate", skip_serializing_if = "Option::is_none")]
    pub consultation_start_date: Option<String>,
    #[serde(rename = "consultationEndDate", skip_serializing_if = "Option::is_none")]
    pub consultation_end_date: Option<String>,
    #[serde(rename = "constructionStartDate", skip_serializing_if = "Option::is_none")]
    pub construction_start_date: Option<String>,
    #[serde(rename = "constructionEndDate", skip_serializing_if = "Option::is_none")]
    pub construction_end_date: Option<String>,
    #[serde(rename = "boroughsBenefited", skip_serializing_if = "Option::is_none")]
    pub boroughs_benefited: Option<Vec<String>>,
    #[serde(rename = "cycleSuperhighwayId", skip_serializing_if = "Option::is_none")]
    pub cycle_superhighway_id: Option<String>,
    #[serde(rename = "phase", skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
    #[serde(rename = "contactName", skip_serializing_if = "Option::is_none")]
    pub contact_name: Option<String>,
    #[serde(rename = "contactEmail", skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
    #[serde(rename = "externalPageUrl", skip_serializing_if = "Option::is_none")]
    pub external_page_url: Option<String>,
    #[serde(rename = "projectSummaryPageUrl", skip_serializing_if = "Option::is_none")]
    pub project_summary_page_url: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadProject {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadProject {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadProject {
            project_id: None,
            scheme_name: None,
            project_name: None,
            project_description: None,
            project_page_url: None,
            consultation_page_url: None,
            consultation_start_date: None,
            consultation_end_date: None,
            construction_start_date: None,
            construction_end_date: None,
            boroughs_benefited: None,
            cycle_superhighway_id: None,
            phase: None,
            contact_name: None,
            contact_email: None,
            external_page_url: None,
            project_summary_page_url: None,
        }
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Phase {
    #[serde(rename = "Unscoped")]
    Unscoped,
    #[serde(rename = "Concept")]
    Concept,
    #[serde(rename = "ConsultationEnded")]
    ConsultationEnded,
    #[serde(rename = "Consultation")]
    Consultation,
    #[serde(rename = "Construction")]
    Construction,
    #[serde(rename = "Complete")]
    Complete,
}

impl Default for Phase {
    fn default() -> Phase {
        Self::Unscoped
    }
}

