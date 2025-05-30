/*
 * Transport for London Unified API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: v1
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;
use serde::{Deserialize, Serialize};
use crate::tflapi::{apis::ResponseContent, models};
use super::{Error, configuration};


/// struct for typed errors of method [`place_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceGetError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_get_at`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceGetAtError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_get_by_geo`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceGetByGeoError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_get_by_type`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceGetByTypeError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_get_overlay`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceGetOverlayError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_get_streets_by_post_code`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceGetStreetsByPostCodeError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_meta_categories`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceMetaCategoriesError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_meta_place_types`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceMetaPlaceTypesError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`place_search`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlaceSearchError {
    UnknownValue(serde_json::Value),
}


pub async fn place_get(configuration: &configuration::Configuration, id: &str, include_children: Option<bool>) -> Result<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace>, Error<PlaceGetError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/{id}", local_var_configuration.base_path, id=crate::tflapi::apis::urlencode(id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = include_children {
        local_var_req_builder = local_var_req_builder.query(&[("includeChildren", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceGetError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_get_at(configuration: &configuration::Configuration, r#type: Vec<String>, lat: &str, lon: &str, location_period_lat: f64, location_period_lon: f64) -> Result<serde_json::Value, Error<PlaceGetAtError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/{type}/At/{lat}/{lon}", local_var_configuration.base_path, type=r#type.join(","));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("lat", &lat.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("lon", &lon.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("location.lat", &location_period_lat.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("location.lon", &location_period_lon.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceGetAtError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_get_by_geo(configuration: &configuration::Configuration, radius: Option<f64>, categories: Option<Vec<String>>, include_children: Option<bool>, r#type: Option<Vec<String>>, active_only: Option<bool>, number_of_places_to_return: Option<i32>, place_geo_period_sw_lat: Option<f64>, place_geo_period_sw_lon: Option<f64>, place_geo_period_ne_lat: Option<f64>, place_geo_period_ne_lon: Option<f64>, place_geo_period_lat: Option<f64>, place_geo_period_lon: Option<f64>) -> Result<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStopPoint>, Error<PlaceGetByGeoError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = radius {
        local_var_req_builder = local_var_req_builder.query(&[("radius", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = categories {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("categories".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("categories", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_str) = include_children {
        local_var_req_builder = local_var_req_builder.query(&[("includeChildren", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = r#type {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("type".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("type", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_str) = active_only {
        local_var_req_builder = local_var_req_builder.query(&[("activeOnly", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = number_of_places_to_return {
        local_var_req_builder = local_var_req_builder.query(&[("numberOfPlacesToReturn", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = place_geo_period_sw_lat {
        local_var_req_builder = local_var_req_builder.query(&[("placeGeo.swLat", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = place_geo_period_sw_lon {
        local_var_req_builder = local_var_req_builder.query(&[("placeGeo.swLon", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = place_geo_period_ne_lat {
        local_var_req_builder = local_var_req_builder.query(&[("placeGeo.neLat", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = place_geo_period_ne_lon {
        local_var_req_builder = local_var_req_builder.query(&[("placeGeo.neLon", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = place_geo_period_lat {
        local_var_req_builder = local_var_req_builder.query(&[("placeGeo.lat", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = place_geo_period_lon {
        local_var_req_builder = local_var_req_builder.query(&[("placeGeo.lon", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceGetByGeoError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_get_by_type(configuration: &configuration::Configuration, types: Vec<String>, active_only: Option<bool>) -> Result<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace>, Error<PlaceGetByTypeError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/Type/{types}", local_var_configuration.base_path, types=types.join(","));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = active_only {
        local_var_req_builder = local_var_req_builder.query(&[("activeOnly", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceGetByTypeError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_get_overlay(configuration: &configuration::Configuration, z: i32, r#type: Vec<String>, width: i32, height: i32, lat: &str, lon: &str, location_period_lat: f64, location_period_lon: f64) -> Result<serde_json::Value, Error<PlaceGetOverlayError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/{type}/overlay/{z}/{lat}/{lon}/{width}/{height}", local_var_configuration.base_path, z=z, type=r#type.join(","), width=width, height=height);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("lat", &lat.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("lon", &lon.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("location.lat", &location_period_lat.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("location.lon", &location_period_lon.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceGetOverlayError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_get_streets_by_post_code(configuration: &configuration::Configuration, postcode: &str, postcode_input_period_postcode: Option<&str>) -> Result<serde_json::Value, Error<PlaceGetStreetsByPostCodeError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/Address/Streets/{postcode}", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("postcode", &postcode.to_string())]);
    if let Some(ref local_var_str) = postcode_input_period_postcode {
        local_var_req_builder = local_var_req_builder.query(&[("postcodeInput.postcode", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceGetStreetsByPostCodeError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_meta_categories(configuration: &configuration::Configuration, ) -> Result<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlaceCategory>, Error<PlaceMetaCategoriesError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/Meta/Categories", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceMetaCategoriesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_meta_place_types(configuration: &configuration::Configuration, ) -> Result<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlaceCategory>, Error<PlaceMetaPlaceTypesError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/Meta/PlaceTypes", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceMetaPlaceTypesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn place_search(configuration: &configuration::Configuration, name: &str, types: Option<Vec<String>>) -> Result<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodPlace>, Error<PlaceSearchError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/Place/Search", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("name", &name.to_string())]);
    if let Some(ref local_var_str) = types {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("types".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("types", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<PlaceSearchError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

