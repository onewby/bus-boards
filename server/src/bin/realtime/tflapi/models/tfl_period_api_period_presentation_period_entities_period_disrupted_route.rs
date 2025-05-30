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

/// TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruptedRoute : keep old RouteSection name so as not to break contract
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruptedRoute {
    /// The Id of the route
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The Id of the Line
    #[serde(rename = "lineId", skip_serializing_if = "Option::is_none")]
    pub line_id: Option<String>,
    /// The route code
    #[serde(rename = "routeCode", skip_serializing_if = "Option::is_none")]
    pub route_code: Option<String>,
    /// Name such as \"72\"
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The co-ordinates of the route's path as a geoJSON lineString
    #[serde(rename = "lineString", skip_serializing_if = "Option::is_none")]
    pub line_string: Option<String>,
    /// Inbound or Outbound
    #[serde(rename = "direction", skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// The name of the Origin StopPoint
    #[serde(rename = "originationName", skip_serializing_if = "Option::is_none")]
    pub origination_name: Option<String>,
    /// The name of the Destination StopPoint
    #[serde(rename = "destinationName", skip_serializing_if = "Option::is_none")]
    pub destination_name: Option<String>,
    #[serde(rename = "via", skip_serializing_if = "Option::is_none")]
    pub via: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSectionNaptanEntrySequence>>,
    /// Whether this represents the entire route section
    #[serde(rename = "isEntireRouteSection", skip_serializing_if = "Option::is_none")]
    pub is_entire_route_section: Option<bool>,
    /// The DateTime that the Service containing this Route is valid until.
    #[serde(rename = "validTo", skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
    /// The DateTime that the Service containing this Route is valid from.
    #[serde(rename = "validFrom", skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(rename = "routeSectionNaptanEntrySequence", skip_serializing_if = "Option::is_none")]
    pub route_section_naptan_entry_sequence: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRouteSectionNaptanEntrySequence>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruptedRoute {
    /// keep old RouteSection name so as not to break contract
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruptedRoute {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodDisruptedRoute {
            id: None,
            line_id: None,
            route_code: None,
            name: None,
            line_string: None,
            direction: None,
            origination_name: None,
            destination_name: None,
            via: None,
            is_entire_route_section: None,
            valid_to: None,
            valid_from: None,
            route_section_naptan_entry_sequence: None,
        }
    }
}

