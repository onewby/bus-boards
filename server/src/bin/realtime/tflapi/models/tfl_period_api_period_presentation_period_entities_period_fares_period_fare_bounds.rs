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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFareBounds {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(rename = "via", skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    #[serde(rename = "routeCode", skip_serializing_if = "Option::is_none")]
    pub route_code: Option<String>,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "operator", skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(rename = "displayOrder", skip_serializing_if = "Option::is_none")]
    pub display_order: Option<i32>,
    #[serde(rename = "isPopularFare", skip_serializing_if = "Option::is_none")]
    pub is_popular_fare: Option<bool>,
    #[serde(rename = "isPopularTravelCard", skip_serializing_if = "Option::is_none")]
    pub is_popular_travel_card: Option<bool>,
    #[serde(rename = "isTour", skip_serializing_if = "Option::is_none")]
    pub is_tour: Option<bool>,
    #[serde(rename = "messages", skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodMessage>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFareBounds {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFareBounds {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodFaresPeriodFareBounds {
            id: None,
            from: None,
            to: None,
            via: None,
            route_code: None,
            description: None,
            display_name: None,
            operator: None,
            display_order: None,
            is_popular_fare: None,
            is_popular_travel_card: None,
            is_tour: None,
            messages: None,
        }
    }
}

