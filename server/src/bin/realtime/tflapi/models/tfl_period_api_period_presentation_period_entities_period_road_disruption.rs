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
pub struct TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruption {
    /// Unique identifier for the road disruption
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// URL to retrieve this road disruption
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Latitude and longitude (WGS84) of the centroid of the disruption, stored in a geoJSON-formatted string.
    #[serde(rename = "point", skip_serializing_if = "Option::is_none")]
    pub point: Option<String>,
    /// A description of the severity of the disruption.
    #[serde(rename = "severity", skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    /// An ordinal of the disruption based on severity, level of interest and corridor.
    #[serde(rename = "ordinal", skip_serializing_if = "Option::is_none")]
    pub ordinal: Option<i32>,
    /// Describes the nature of disruption e.g. Traffic Incidents, Works
    #[serde(rename = "category", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Describes the sub-category of disruption e.g. Collapsed Manhole, Abnormal Load
    #[serde(rename = "subCategory", skip_serializing_if = "Option::is_none")]
    pub sub_category: Option<String>,
    /// Full text of comments describing the disruption, including details of any road closures and diversions, where appropriate.
    #[serde(rename = "comments", skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Text of the most recent update from the LSTCC on the state of the               disruption, including the current traffic impact and any advice to               road users.
    #[serde(rename = "currentUpdate", skip_serializing_if = "Option::is_none")]
    pub current_update: Option<String>,
    /// The time when the last CurrentUpdate description was recorded,               or null if no CurrentUpdate has been applied.
    #[serde(rename = "currentUpdateDateTime", skip_serializing_if = "Option::is_none")]
    pub current_update_date_time: Option<String>,
    /// The Ids of affected corridors, if any.
    #[serde(rename = "corridorIds", skip_serializing_if = "Option::is_none")]
    pub corridor_ids: Option<Vec<String>>,
    /// The date and time which the disruption started. For a planned disruption (i.e. planned road works) this date will be in the future.              For unplanned disruptions, this will default to the date on which the disruption was first recorded, but may be adjusted by the operator.
    #[serde(rename = "startDateTime", skip_serializing_if = "Option::is_none")]
    pub start_date_time: Option<String>,
    /// The date and time on which the disruption ended. For planned disruptions, this date will have a valid value. For unplanned               disruptions in progress, this field will be omitted.
    #[serde(rename = "endDateTime", skip_serializing_if = "Option::is_none")]
    pub end_date_time: Option<String>,
    /// The date and time on which the disruption was last modified in the system. This information can reliably be used by a developer to quickly              compare two instances of the same disruption to determine if it has been changed.
    #[serde(rename = "lastModifiedTime", skip_serializing_if = "Option::is_none")]
    pub last_modified_time: Option<String>,
    /// This describes the level of potential impact on traffic operations of the disruption.               High = e.g. a one-off disruption on a major or high profile route which will require a high level of operational attention               Medium = This is the default value               Low = e.g. a frequently occurring disruption which is well known
    #[serde(rename = "levelOfInterest", skip_serializing_if = "Option::is_none")]
    pub level_of_interest: Option<String>,
    /// Main road name / number (borough) or preset area name where the disruption is located. This might be useful for a map popup where space is limited.
    #[serde(rename = "location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// This describes the status of the disruption.                Active = currently in progress               Active Long Term = currently in progress and long term              Scheduled = scheduled to start within the next 180 days              Recurring Works = planned maintenance works that follow a regular routine or pattern and whose next occurrence is to start within the next 180 days.              Recently Cleared = recently cleared in the last 24 hours              Note that the status of Scheduled or Recurring Works disruptions will change to Active when they start, and will change status again when they end.
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "geography", skip_serializing_if = "Option::is_none")]
    pub geography: Option<Box<models::SystemPeriodDataPeriodSpatialPeriodDbGeography>>,
    #[serde(rename = "geometry", skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Box<models::SystemPeriodDataPeriodSpatialPeriodDbGeography>>,
    /// A collection of zero or more streets affected by the disruption.
    #[serde(rename = "streets", skip_serializing_if = "Option::is_none")]
    pub streets: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodStreet>>,
    /// True if the disruption is planned on a future date that is open to change
    #[serde(rename = "isProvisional", skip_serializing_if = "Option::is_none")]
    pub is_provisional: Option<bool>,
    /// True if any of the affected Streets have a \"Full Closure\" status, false otherwise. A RoadDisruption that has HasClosures is considered a               Severe or Serious disruption for severity filtering purposes.
    #[serde(rename = "hasClosures", skip_serializing_if = "Option::is_none")]
    pub has_closures: Option<bool>,
    /// The text of any associated link
    #[serde(rename = "linkText", skip_serializing_if = "Option::is_none")]
    pub link_text: Option<String>,
    /// The url of any associated link
    #[serde(rename = "linkUrl", skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    #[serde(rename = "roadProject", skip_serializing_if = "Option::is_none")]
    pub road_project: Option<Box<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadProject>>,
    /// TDM Additional properties
    #[serde(rename = "publishStartDate", skip_serializing_if = "Option::is_none")]
    pub publish_start_date: Option<String>,
    #[serde(rename = "publishEndDate", skip_serializing_if = "Option::is_none")]
    pub publish_end_date: Option<String>,
    #[serde(rename = "timeFrame", skip_serializing_if = "Option::is_none")]
    pub time_frame: Option<String>,
    #[serde(rename = "roadDisruptionLines", skip_serializing_if = "Option::is_none")]
    pub road_disruption_lines: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionLine>>,
    #[serde(rename = "roadDisruptionImpactAreas", skip_serializing_if = "Option::is_none")]
    pub road_disruption_impact_areas: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionImpactArea>>,
    #[serde(rename = "recurringSchedules", skip_serializing_if = "Option::is_none")]
    pub recurring_schedules: Option<Vec<models::TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruptionSchedule>>,
}

impl TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruption {
    pub fn new() -> TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruption {
        TflPeriodApiPeriodPresentationPeriodEntitiesPeriodRoadDisruption {
            id: None,
            url: None,
            point: None,
            severity: None,
            ordinal: None,
            category: None,
            sub_category: None,
            comments: None,
            current_update: None,
            current_update_date_time: None,
            corridor_ids: None,
            start_date_time: None,
            end_date_time: None,
            last_modified_time: None,
            level_of_interest: None,
            location: None,
            status: None,
            geography: None,
            geometry: None,
            streets: None,
            is_provisional: None,
            has_closures: None,
            link_text: None,
            link_url: None,
            road_project: None,
            publish_start_date: None,
            publish_end_date: None,
            time_frame: None,
            road_disruption_lines: None,
            road_disruption_impact_areas: None,
            recurring_schedules: None,
        }
    }
}

