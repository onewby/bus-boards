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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace {
    /// A unique identifier.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The unique location of this resource.
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A human readable name.
    #[serde(rename = "commonName", skip_serializing_if = "Option::is_none")]
    pub common_name: Option<String>,
    /// The distance of the place from its search point, if this is the result              of a geographical search, otherwise zero.
    #[serde(rename = "distance", skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    /// The type of Place. See /Place/Meta/placeTypes for possible values.
    #[serde(rename = "placeType", skip_serializing_if = "Option::is_none")]
    pub place_type: Option<String>,
    /// A bag of additional key/value pairs with extra information about this place.
    #[serde(rename = "additionalProperties", skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodAdditionalProperties>>,
    #[serde(rename = "children", skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace>>,
    #[serde(rename = "childrenUrls", skip_serializing_if = "Option::is_none")]
    pub children_urls: Option<Vec<String>>,
    /// WGS84 latitude of the location.
    #[serde(rename = "lat", skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    /// WGS84 longitude of the location.
    #[serde(rename = "lon", skip_serializing_if = "Option::is_none")]
    pub lon: Option<f64>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace {
            id: None,
            url: None,
            common_name: None,
            distance: None,
            place_type: None,
            additional_properties: None,
            children: None,
            children_urls: None,
            lat: None,
            lon: None,
        }
    }
}

