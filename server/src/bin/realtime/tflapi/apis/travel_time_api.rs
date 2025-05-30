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


/// struct for typed errors of method [`travel_time_get_compare_overlay`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TravelTimeGetCompareOverlayError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`travel_time_get_overlay`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TravelTimeGetOverlayError {
    UnknownValue(serde_json::Value),
}


pub async fn travel_time_get_compare_overlay(configuration: &configuration::Configuration, z: i32, pin_lat: f64, pin_lon: f64, map_center_lat: f64, map_center_lon: f64, scenario_title: &str, time_of_day_id: &str, mode_id: &str, width: i32, height: i32, direction: &str, travel_time_interval: i32, compare_type: &str, compare_value: &str) -> Result<serde_json::Value, Error<TravelTimeGetCompareOverlayError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/TravelTimes/compareOverlay/{z}/mapcenter/{mapCenterLat}/{mapCenterLon}/pinlocation/{pinLat}/{pinLon}/dimensions/{width}/{height}", local_var_configuration.base_path, z=z, pinLat=pin_lat, pinLon=pin_lon, mapCenterLat=map_center_lat, mapCenterLon=map_center_lon, width=width, height=height);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("scenarioTitle", &scenario_title.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("timeOfDayId", &time_of_day_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("modeId", &mode_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("direction", &direction.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("travelTimeInterval", &travel_time_interval.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("compareType", &compare_type.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("compareValue", &compare_value.to_string())]);
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
        let local_var_entity: Option<TravelTimeGetCompareOverlayError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

pub async fn travel_time_get_overlay(configuration: &configuration::Configuration, z: i32, pin_lat: f64, pin_lon: f64, map_center_lat: f64, map_center_lon: f64, scenario_title: &str, time_of_day_id: &str, mode_id: &str, width: i32, height: i32, direction: &str, travel_time_interval: i32) -> Result<serde_json::Value, Error<TravelTimeGetOverlayError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/TravelTimes/overlay/{z}/mapcenter/{mapCenterLat}/{mapCenterLon}/pinlocation/{pinLat}/{pinLon}/dimensions/{width}/{height}", local_var_configuration.base_path, z=z, pinLat=pin_lat, pinLon=pin_lon, mapCenterLat=map_center_lat, mapCenterLon=map_center_lon, width=width, height=height);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("scenarioTitle", &scenario_title.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("timeOfDayId", &time_of_day_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("modeId", &mode_id.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("direction", &direction.to_string())]);
    local_var_req_builder = local_var_req_builder.query(&[("travelTimeInterval", &travel_time_interval.to_string())]);
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
        let local_var_entity: Option<TravelTimeGetOverlayError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

