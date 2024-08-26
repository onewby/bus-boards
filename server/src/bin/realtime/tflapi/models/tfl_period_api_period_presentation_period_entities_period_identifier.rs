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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodIdentifier {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "uri", skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "fullName", skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "crowding", skip_serializing_if = "Option::is_none")]
    pub crowding: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodCrowding>>,
    #[serde(rename = "routeType", skip_serializing_if = "Option::is_none")]
    pub route_type: Option<RouteType>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(rename = "motType", skip_serializing_if = "Option::is_none")]
    pub mot_type: Option<String>,
    #[serde(rename = "network", skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodIdentifier {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodIdentifier {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodIdentifier {
            id: None,
            name: None,
            uri: None,
            full_name: None,
            r#type: None,
            crowding: None,
            route_type: None,
            status: None,
            mot_type: None,
            network: None,
        }
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum RouteType {
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "All")]
    All,
    #[serde(rename = "Cycle Superhighways")]
    CycleSuperhighways,
    #[serde(rename = "Quietways")]
    Quietways,
    #[serde(rename = "Cycleways")]
    Cycleways,
    #[serde(rename = "Mini-Hollands")]
    MiniHollands,
    #[serde(rename = "Central London Grid")]
    CentralLondonGrid,
    #[serde(rename = "Streetspace Route")]
    StreetspaceRoute,
}

impl Default for RouteType {
    fn default() -> RouteType {
        Self::Unknown
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "All")]
    All,
    #[serde(rename = "Open")]
    Open,
    #[serde(rename = "In Progress")]
    InProgress,
    #[serde(rename = "Planned")]
    Planned,
    #[serde(rename = "Planned - Subject to feasibility and consultation.")]
    PlannedSubjectToFeasibilityAndConsultationPeriod,
    #[serde(rename = "Not Open")]
    NotOpen,
}

impl Default for Status {
    fn default() -> Status {
        Self::Unknown
    }
}

