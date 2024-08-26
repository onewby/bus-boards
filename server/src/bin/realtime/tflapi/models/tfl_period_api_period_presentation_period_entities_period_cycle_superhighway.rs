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

/// TflPeriodApiPeriodPresentationPeriodEntitiesPeriodCycleSuperhighway : 
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodCycleSuperhighway {
    /// The Id
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The long label to show on maps when zoomed in
    #[serde(rename = "label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The short label to show on maps
    #[serde(rename = "labelShort", skip_serializing_if = "Option::is_none")]
    pub label_short: Option<String>,
    #[serde(rename = "geography", skip_serializing_if = "Option::is_none")]
    pub geography: Option<Box<models::SystemPeriodDataPeriodSpatialPeriodDbGeography>>,
    /// True if the route is split into segments
    #[serde(rename = "segmented", skip_serializing_if = "Option::is_none")]
    pub segmented: Option<bool>,
    /// When the data was last updated
    #[serde(rename = "modified", skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// Cycle route status i.e Proposed, Existing etc
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// Type of cycle route e.g CycleSuperhighways, Quietways, MiniHollands etc
    #[serde(rename = "routeType", skip_serializing_if = "Option::is_none")]
    pub route_type: Option<RouteType>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodCycleSuperhighway {
    /// 
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodCycleSuperhighway {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodCycleSuperhighway {
            id: None,
            label: None,
            label_short: None,
            geography: None,
            segmented: None,
            modified: None,
            status: None,
            route_type: None,
        }
    }
}
/// Cycle route status i.e Proposed, Existing etc
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
/// Type of cycle route e.g CycleSuperhighways, Quietways, MiniHollands etc
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
