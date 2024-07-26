//! THIS IS A GENERATED FILE!
//! Take care when hand editing. Changes will be lost during subsequent runs of the code generator.
//!
//! version: 0.1.10
//!

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_local_definitions)]

use yaserde::{YaSerialize, YaDeserialize};
use std::io::{Read, Write};
use log::{warn, debug, trace};
use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::api::darwin::types::AccessToken;

pub const SOAP_ENCODING: &str = "http://www.w3.org/2003/05/soap-encoding";

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
    rename = "Header",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soapenv",
)]
pub struct Header {
    #[yaserde(rename = "AccessToken")]
    pub access_token: AccessToken,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
    rename = "Fault",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soapenv",
)]
pub struct SoapFault {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: Option<String>,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
}

impl std::error::Error for SoapFault {}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.fault_code, &self.fault_string) {
            (None, None) => Ok(()),
            (None, Some(fault_string)) => f.write_str(fault_string),
            (Some(fault_code), None) => f.write_str(fault_code),
            (Some(fault_code), Some(fault_string)) => {
                f.write_str(fault_code)?;
                f.write_str(": ")?;
                f.write_str(fault_string)
            }
        }
    }
}

pub type SoapResponse = Result<(reqwest::StatusCode, String), reqwest::Error>;

pub mod messages {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetDepartureBoardSoapIn",
    )]
    pub struct GetDepartureBoardSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetDepartureBoardRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetDepartureBoardSoapOut",
    )]
    pub struct GetDepartureBoardSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetDepartureBoardResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrivalBoardSoapIn",
    )]
    pub struct GetArrivalBoardSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrivalBoardRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrivalBoardSoapOut",
    )]
    pub struct GetArrivalBoardSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrivalBoardResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrivalDepartureBoardSoapIn",
    )]
    pub struct GetArrivalDepartureBoardSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrivalDepartureBoardRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrivalDepartureBoardSoapOut",
    )]
    pub struct GetArrivalDepartureBoardSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrivalDepartureBoardResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetServiceDetailsSoapIn",
    )]
    pub struct GetServiceDetailsSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetServiceDetailsRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetServiceDetailsSoapOut",
    )]
    pub struct GetServiceDetailsSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetServiceDetailsResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetDepBoardWithDetailsSoapIn",
    )]
    pub struct GetDepBoardWithDetailsSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetDepBoardWithDetailsRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetDepBoardWithDetailsSoapOut",
    )]
    pub struct GetDepBoardWithDetailsSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetDepBoardWithDetailsResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrBoardWithDetailsSoapIn",
    )]
    pub struct GetArrBoardWithDetailsSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrBoardWithDetailsRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrBoardWithDetailsSoapOut",
    )]
    pub struct GetArrBoardWithDetailsSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrBoardWithDetailsResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrDepBoardWithDetailsSoapIn",
    )]
    pub struct GetArrDepBoardWithDetailsSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrDepBoardWithDetailsRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetArrDepBoardWithDetailsSoapOut",
    )]
    pub struct GetArrDepBoardWithDetailsSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetArrDepBoardWithDetailsResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetNextDeparturesSoapIn",
    )]
    pub struct GetNextDeparturesSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetNextDeparturesRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetNextDeparturesSoapOut",
    )]
    pub struct GetNextDeparturesSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetNextDeparturesResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetNextDeparturesWithDetailsSoapIn",
    )]
    pub struct GetNextDeparturesWithDetailsSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetNextDeparturesWithDetailsRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetNextDeparturesWithDetailsSoapOut",
    )]
    pub struct GetNextDeparturesWithDetailsSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetNextDeparturesWithDetailsResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetFastestDeparturesSoapIn",
    )]
    pub struct GetFastestDeparturesSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetFastestDeparturesRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetFastestDeparturesSoapOut",
    )]
    pub struct GetFastestDeparturesSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetFastestDeparturesResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetFastestDeparturesWithDetailsSoapIn",
    )]
    pub struct GetFastestDeparturesWithDetailsSoapIn {
        #[yaserde(flatten, default)]
        pub parameters: types::GetFastestDeparturesWithDetailsRequest,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetFastestDeparturesWithDetailsSoapOut",
    )]
    pub struct GetFastestDeparturesWithDetailsSoapOut {
        #[yaserde(flatten, default)]
        pub parameters: types::GetFastestDeparturesWithDetailsResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "AccessTokenMessage",
    )]
    pub struct AccessTokenMessage {
        #[yaserde(flatten, default)]
        pub access_token: types::AccessToken,
    }
}

pub mod types {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "AccessToken",
        namespace = "tns: http://thalesgroup.com/RTTI/2013-11-28/Token/types",
        prefix = "tns",
    )]
    pub struct AccessToken {
        #[yaserde(rename = "TokenValue", prefix = "tns", default)]
        pub token_value: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CRSType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct Crstype {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "LocationNameType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct LocationNameType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "TOCName",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct Tocname {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "TOCCode",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct Toccode {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "PlatformType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct PlatformType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "FilterType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct FilterType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct ServiceType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "TiplocType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct TiplocType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "UIDType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct Uidtype {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "RIDType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct Ridtype {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "RSIDType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct Rsidtype {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "TrainIDType",
        namespace = "nsi1: http://thalesgroup.com/RTTI/2007-10-10/ldb/commontypes",
        prefix = "nsi1",
    )]
    pub struct TrainIDType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "TimeType",
        namespace = "nsi2: http://thalesgroup.com/RTTI/2015-11-27/ldb/commontypes",
        prefix = "nsi2",
    )]
    pub struct TimeType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceIDType",
        namespace = "nsi2: http://thalesgroup.com/RTTI/2015-11-27/ldb/commontypes",
        prefix = "nsi2",
    )]
    pub struct ServiceIDType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "LoadingValue",
        namespace = "nsi7: http://thalesgroup.com/RTTI/2017-02-02/ldb/commontypes",
        prefix = "nsi7",
    )]
    pub struct LoadingValue {
        #[yaserde(default)]
        pub body: u32,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CoachNumberType",
        namespace = "nsi7: http://thalesgroup.com/RTTI/2017-02-02/ldb/commontypes",
        prefix = "nsi7",
    )]
    pub struct CoachNumberType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CoachClassType",
        namespace = "nsi7: http://thalesgroup.com/RTTI/2017-02-02/ldb/commontypes",
        prefix = "nsi7",
    )]
    pub struct CoachClassType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ToiletType",
        namespace = "nsi8: http://thalesgroup.com/RTTI/2017-10-01/ldb/commontypes",
        prefix = "nsi8",
    )]
    pub struct ToiletType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ToiletStatus",
        namespace = "nsi8: http://thalesgroup.com/RTTI/2017-10-01/ldb/commontypes",
        prefix = "nsi8",
    )]
    pub struct ToiletStatus {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ToiletAvailabilityType",
        namespace = "nsi8: http://thalesgroup.com/RTTI/2017-10-01/ldb/commontypes",
        prefix = "nsi8",
    )]
    pub struct ToiletAvailabilityType {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "AdhocAlertTextType",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct AdhocAlertTextType {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceLocation",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ServiceLocation {
        #[yaserde(rename = "locationName", prefix = "nsi9", default)]
        pub location_name: LocationNameType,
        #[yaserde(rename = "crs", prefix = "nsi9", default)]
        pub crs: Crstype,
        #[yaserde(rename = "via", prefix = "nsi9", default)]
        pub via: Option<String>,
        #[yaserde(rename = "futureChangeTo", prefix = "nsi9", default)]
        pub future_change_to: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfServiceLocations",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ArrayOfServiceLocations {
        #[yaserde(rename = "location", prefix = "nsi9", default)]
        pub location: Vec<ServiceLocation>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceItem",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ServiceItem {
        #[yaserde(rename = "origin", prefix = "nsi9", default)]
        pub origin: Option<ArrayOfServiceLocations>,
        #[yaserde(rename = "destination", prefix = "nsi9", default)]
        pub destination: Option<ArrayOfServiceLocations>,
        #[yaserde(rename = "currentOrigins", prefix = "nsi9", default)]
        pub current_origins: Option<ArrayOfServiceLocations>,
        #[yaserde(rename = "currentDestinations", prefix = "nsi9", default)]
        pub current_destinations: Option<ArrayOfServiceLocations>,
        #[yaserde(rename = "sta", prefix = "nsi9", default)]
        pub sta: Option<TimeType>,
        #[yaserde(rename = "eta", prefix = "nsi9", default)]
        pub eta: Option<TimeType>,
        #[yaserde(rename = "std", prefix = "nsi9", default)]
        pub std: Option<TimeType>,
        #[yaserde(rename = "etd", prefix = "nsi9", default)]
        pub etd: Option<TimeType>,
        #[yaserde(rename = "platform", prefix = "nsi9", default)]
        pub platform: Option<PlatformType>,
        #[yaserde(rename = "operator", prefix = "nsi9", default)]
        pub operator: Tocname,
        #[yaserde(rename = "operatorCode", prefix = "nsi9", default)]
        pub operator_code: Toccode,
        #[yaserde(rename = "isCircularRoute", prefix = "nsi9", default)]
        pub is_circular_route: Option<bool>,
        #[yaserde(rename = "serviceID", prefix = "nsi9", default)]
        pub service_id: ServiceIDType,
        #[yaserde(rename = "adhocAlerts", prefix = "nsi9", default)]
        pub adhoc_alerts: Option<ArrayOfAdhocAlert>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfServiceItems",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ArrayOfServiceItems {
        #[yaserde(rename = "service", prefix = "nsi9", default)]
        pub service: Vec<ServiceItem>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "NRCCMessage",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct Nrccmessage {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfNRCCMessages",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ArrayOfNRCCMessages {
        #[yaserde(rename = "message", prefix = "nsi9", default)]
        pub message: Vec<Nrccmessage>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CallingPoint",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct CallingPoint {
        #[yaserde(rename = "locationName", prefix = "nsi9", default)]
        pub location_name: LocationNameType,
        #[yaserde(rename = "crs", prefix = "nsi9", default)]
        pub crs: Crstype,
        #[yaserde(rename = "st", prefix = "nsi9", default)]
        pub st: Option<TimeType>,
        #[yaserde(rename = "et", prefix = "nsi9", default)]
        pub et: Option<TimeType>,
        #[yaserde(rename = "at", prefix = "nsi9", default)]
        pub at: Option<TimeType>,
        #[yaserde(rename = "adhocAlerts", prefix = "nsi9", default)]
        pub adhoc_alerts: Option<ArrayOfAdhocAlert>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfCallingPoints",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ArrayOfCallingPoints {
        #[yaserde(rename = "serviceType", attribute)]
        pub service_type: ServiceType,
        #[yaserde(rename = "serviceChangeRequired", attribute)]
        pub service_change_required: bool,
        #[yaserde(rename = "callingPoint", prefix = "nsi9", default)]
        pub calling_point: Vec<CallingPoint>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfArrayOfCallingPoints",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ArrayOfArrayOfCallingPoints {
        #[yaserde(rename = "callingPointList", prefix = "nsi9", default)]
        pub calling_point_list: Vec<ArrayOfCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "StationBoard",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct StationBoard {
        #[yaserde(rename = "generatedAt", prefix = "nsi9", default)]
        pub generated_at: String,
        #[yaserde(rename = "locationName", prefix = "nsi9", default)]
        pub location_name: LocationNameType,
        #[yaserde(rename = "crs", prefix = "nsi9", default)]
        pub crs: Crstype,
        #[yaserde(rename = "filterLocationName", prefix = "nsi9", default)]
        pub filter_location_name: Option<LocationNameType>,
        #[yaserde(rename = "filtercrs", prefix = "nsi9", default)]
        pub filtercrs: Option<Crstype>,
        #[yaserde(rename = "filterType", prefix = "nsi9", default)]
        pub filter_type: Option<FilterType>,
        #[yaserde(rename = "nrccMessages", prefix = "nsi9", default)]
        pub nrcc_messages: Option<ArrayOfNRCCMessages>,
        #[yaserde(rename = "platformAvailable", prefix = "nsi9", default)]
        pub platform_available: Option<bool>,
        #[yaserde(rename = "areServicesAvailable", prefix = "nsi9", default)]
        pub are_services_available: Option<bool>,
        #[yaserde(rename = "trainServices", prefix = "nsi9", default)]
        pub train_services: Option<ArrayOfServiceItems>,
        #[yaserde(rename = "busServices", prefix = "nsi9", default)]
        pub bus_services: Option<ArrayOfServiceItems>,
        #[yaserde(rename = "ferryServices", prefix = "nsi9", default)]
        pub ferry_services: Option<ArrayOfServiceItems>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfAdhocAlert",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ArrayOfAdhocAlert {
        #[yaserde(rename = "adhocAlertText", prefix = "nsi9", default)]
        pub adhoc_alert_text: Vec<AdhocAlertTextType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceDetails",
        namespace = "nsi9: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
        prefix = "nsi9",
    )]
    pub struct ServiceDetails {
        #[yaserde(rename = "generatedAt", prefix = "nsi9", default)]
        pub generated_at: String,
        #[yaserde(rename = "serviceType", prefix = "nsi9", default)]
        pub service_type: ServiceType,
        #[yaserde(rename = "locationName", prefix = "nsi9", default)]
        pub location_name: LocationNameType,
        #[yaserde(rename = "crs", prefix = "nsi9", default)]
        pub crs: Crstype,
        #[yaserde(rename = "operator", prefix = "nsi9", default)]
        pub operator: Tocname,
        #[yaserde(rename = "operatorCode", prefix = "nsi9", default)]
        pub operator_code: Toccode,
        #[yaserde(rename = "isCancelled", prefix = "nsi9", default)]
        pub is_cancelled: Option<bool>,
        #[yaserde(rename = "disruptionReason", prefix = "nsi9", default)]
        pub disruption_reason: Option<String>,
        #[yaserde(rename = "overdueMessage", prefix = "nsi9", default)]
        pub overdue_message: Option<String>,
        #[yaserde(rename = "platform", prefix = "nsi9", default)]
        pub platform: Option<PlatformType>,
        #[yaserde(rename = "sta", prefix = "nsi9", default)]
        pub sta: Option<TimeType>,
        #[yaserde(rename = "eta", prefix = "nsi9", default)]
        pub eta: Option<TimeType>,
        #[yaserde(rename = "ata", prefix = "nsi9", default)]
        pub ata: Option<TimeType>,
        #[yaserde(rename = "std", prefix = "nsi9", default)]
        pub std: Option<TimeType>,
        #[yaserde(rename = "etd", prefix = "nsi9", default)]
        pub etd: Option<TimeType>,
        #[yaserde(rename = "atd", prefix = "nsi9", default)]
        pub atd: Option<TimeType>,
        #[yaserde(rename = "adhocAlerts", prefix = "nsi9", default)]
        pub adhoc_alerts: Option<ArrayOfAdhocAlert>,
        #[yaserde(rename = "previousCallingPoints", prefix = "nsi9", default)]
        pub previous_calling_points: Option<ArrayOfArrayOfCallingPoints>,
        #[yaserde(rename = "subsequentCallingPoints", prefix = "nsi9", default)]
        pub subsequent_calling_points: Option<ArrayOfArrayOfCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "StationBoardWithDetails",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct StationBoardWithDetails {
        #[yaserde(flatten, default)]
        pub base_station_board: BaseStationBoard,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
        #[yaserde(rename = "trainServices", prefix = "nsi11", default)]
        pub train_services: Option<ArrayOfServiceItemsWithCallingPoints>,
        #[yaserde(rename = "busServices", prefix = "nsi11", default)]
        pub bus_services: Option<ArrayOfServiceItemsWithCallingPoints>,
        #[yaserde(rename = "ferryServices", prefix = "nsi11", default)]
        pub ferry_services: Option<ArrayOfServiceItemsWithCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DeparturesBoard",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct DeparturesBoard {
        #[yaserde(flatten, default)]
        pub base_station_board: BaseStationBoard,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
        #[yaserde(rename = "departures", prefix = "nsi11", default)]
        pub departures: ArrayOfDepartureItems,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DeparturesBoardWithDetails",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct DeparturesBoardWithDetails {
        #[yaserde(flatten, default)]
        pub base_station_board: BaseStationBoard,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
        #[yaserde(rename = "departures", prefix = "nsi11", default)]
        pub departures: ArrayOfDepartureItemsWithCallingPoints,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfServiceItemsWithCallingPoints",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct ArrayOfServiceItemsWithCallingPoints {
        #[yaserde(rename = "service", prefix = "nsi11", default)]
        pub service: Vec<ServiceItemWithCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfDepartureItems",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct ArrayOfDepartureItems {
        #[yaserde(rename = "destination", prefix = "nsi11", default)]
        pub destination: Vec<DepartureItem>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfDepartureItemsWithCallingPoints",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct ArrayOfDepartureItemsWithCallingPoints {
        #[yaserde(rename = "destination", prefix = "nsi11", default)]
        pub destination: Vec<DepartureItemWithCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "BaseStationBoard",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct BaseStationBoard {
        #[yaserde(rename = "generatedAt", prefix = "nsi11", default)]
        pub generated_at: String,
        #[yaserde(rename = "locationName", prefix = "nsi11", default)]
        pub location_name: LocationNameType,
        #[yaserde(rename = "crs", prefix = "nsi11", default)]
        pub crs: Crstype,
        #[yaserde(rename = "filterLocationName", prefix = "nsi11", default)]
        pub filter_location_name: Option<LocationNameType>,
        #[yaserde(rename = "filtercrs", prefix = "nsi11", default)]
        pub filtercrs: Option<Crstype>,
        #[yaserde(rename = "filterType", prefix = "nsi11", default)]
        pub filter_type: Option<FilterType>,
        #[yaserde(rename = "nrccMessages", prefix = "nsi11", default)]
        pub nrcc_messages: Option<ArrayOfNRCCMessages>,
        #[yaserde(rename = "platformAvailable", prefix = "nsi11", default)]
        pub platform_available: Option<bool>,
        #[yaserde(rename = "areServicesAvailable", prefix = "nsi11", default)]
        pub are_services_available: Option<bool>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "BaseServiceItem",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct BaseServiceItem {
        #[yaserde(rename = "sta", prefix = "nsi11", default)]
        pub sta: Option<TimeType>,
        #[yaserde(rename = "eta", prefix = "nsi11", default)]
        pub eta: Option<TimeType>,
        #[yaserde(rename = "std", prefix = "nsi11", default)]
        pub std: Option<TimeType>,
        #[yaserde(rename = "etd", prefix = "nsi11", default)]
        pub etd: Option<TimeType>,
        #[yaserde(rename = "platform", prefix = "nsi11", default)]
        pub platform: Option<PlatformType>,
        #[yaserde(rename = "operator", prefix = "nsi11", default)]
        pub operator: Tocname,
        #[yaserde(rename = "operatorCode", prefix = "nsi11", default)]
        pub operator_code: Toccode,
        #[yaserde(rename = "isCircularRoute", prefix = "nsi11", default)]
        pub is_circular_route: Option<bool>,
        #[yaserde(rename = "isCancelled", prefix = "nsi11", default)]
        pub is_cancelled: Option<bool>,
        #[yaserde(rename = "filterLocationCancelled", prefix = "nsi11", default)]
        pub filter_location_cancelled: Option<bool>,
        #[yaserde(rename = "serviceType", prefix = "nsi11", default)]
        pub service_type: ServiceType,
        #[yaserde(rename = "length", prefix = "nsi11", default)]
        pub length: Option<u16>,
        #[yaserde(rename = "detachFront", prefix = "nsi11", default)]
        pub detach_front: Option<bool>,
        #[yaserde(rename = "isReverseFormation", prefix = "nsi11", default)]
        pub is_reverse_formation: Option<bool>,
        #[yaserde(rename = "cancelReason", prefix = "nsi11", default)]
        pub cancel_reason: Option<String>,
        #[yaserde(rename = "delayReason", prefix = "nsi11", default)]
        pub delay_reason: Option<String>,
        #[yaserde(rename = "serviceID", prefix = "nsi11", default)]
        pub service_id: ServiceIDType,
        #[yaserde(rename = "adhocAlerts", prefix = "nsi11", default)]
        pub adhoc_alerts: Option<ArrayOfAdhocAlert>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceItemWithCallingPoints",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct ServiceItemWithCallingPoints {
        #[yaserde(flatten, default)]
        pub service_item: ServiceItem,
        #[yaserde(prefix = "xsi", rename = "type", attribute)]
        pub xsi_type: String,
        #[yaserde(rename = "previousCallingPoints", prefix = "nsi11", default)]
        pub previous_calling_points: Option<ArrayOfArrayOfCallingPoints>,
        #[yaserde(rename = "subsequentCallingPoints", prefix = "nsi11", default)]
        pub subsequent_calling_points: Option<ArrayOfArrayOfCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DepartureItem",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct DepartureItem {
        #[yaserde(rename = "crs", attribute)]
        pub crs: Crstype,
        #[yaserde(rename = "service", prefix = "nsi11", default)]
        pub service: Option<ServiceItem>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DepartureItemWithCallingPoints",
        namespace = "nsi11: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
        prefix = "nsi11",
    )]
    pub struct DepartureItemWithCallingPoints {
        #[yaserde(rename = "crs", attribute)]
        pub crs: Crstype,
        #[yaserde(rename = "service", prefix = "nsi11", default)]
        pub service: Option<ServiceItemWithCallingPoints>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "BaseServiceDetails",
        namespace = "nsi24: http://thalesgroup.com/RTTI/2017-10-01/ldb/types",
        prefix = "nsi24",
    )]
    pub struct BaseServiceDetails {
        #[yaserde(rename = "generatedAt", prefix = "nsi24", default)]
        pub generated_at: String,
        #[yaserde(rename = "serviceType", prefix = "nsi24", default)]
        pub service_type: ServiceType,
        #[yaserde(rename = "locationName", prefix = "nsi24", default)]
        pub location_name: LocationNameType,
        #[yaserde(rename = "crs", prefix = "nsi24", default)]
        pub crs: Crstype,
        #[yaserde(rename = "operator", prefix = "nsi24", default)]
        pub operator: Tocname,
        #[yaserde(rename = "operatorCode", prefix = "nsi24", default)]
        pub operator_code: Toccode,
        #[yaserde(rename = "rsid", prefix = "nsi24", default)]
        pub rsid: Option<Rsidtype>,
        #[yaserde(rename = "isCancelled", prefix = "nsi24", default)]
        pub is_cancelled: Option<bool>,
        #[yaserde(rename = "cancelReason", prefix = "nsi24", default)]
        pub cancel_reason: Option<String>,
        #[yaserde(rename = "delayReason", prefix = "nsi24", default)]
        pub delay_reason: Option<String>,
        #[yaserde(rename = "overdueMessage", prefix = "nsi24", default)]
        pub overdue_message: Option<String>,
        #[yaserde(rename = "length", prefix = "nsi24", default)]
        pub length: Option<u16>,
        #[yaserde(rename = "detachFront", prefix = "nsi24", default)]
        pub detach_front: Option<bool>,
        #[yaserde(rename = "isReverseFormation", prefix = "nsi24", default)]
        pub is_reverse_formation: Option<bool>,
        #[yaserde(rename = "platform", prefix = "nsi24", default)]
        pub platform: Option<PlatformType>,
        #[yaserde(rename = "sta", prefix = "nsi24", default)]
        pub sta: Option<TimeType>,
        #[yaserde(rename = "eta", prefix = "nsi24", default)]
        pub eta: Option<TimeType>,
        #[yaserde(rename = "ata", prefix = "nsi24", default)]
        pub ata: Option<TimeType>,
        #[yaserde(rename = "std", prefix = "nsi24", default)]
        pub std: Option<TimeType>,
        #[yaserde(rename = "etd", prefix = "nsi24", default)]
        pub etd: Option<TimeType>,
        #[yaserde(rename = "atd", prefix = "nsi24", default)]
        pub atd: Option<TimeType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "FormationData",
        namespace = "nsi24: http://thalesgroup.com/RTTI/2017-10-01/ldb/types",
        prefix = "nsi24",
    )]
    pub struct FormationData {
        #[yaserde(rename = "avgLoading", prefix = "nsi24", default)]
        pub avg_loading: Option<LoadingValue>,
        #[yaserde(rename = "coaches", prefix = "nsi24", default)]
        pub coaches: Option<ArrayOfCoaches>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CoachData",
        namespace = "nsi24: http://thalesgroup.com/RTTI/2017-10-01/ldb/types",
        prefix = "nsi24",
    )]
    pub struct CoachData {
        #[yaserde(rename = "number", attribute)]
        pub number: CoachNumberType,
        #[yaserde(rename = "coachClass", prefix = "nsi24", default)]
        pub coach_class: Option<CoachClassType>,
        #[yaserde(rename = "toilet", prefix = "nsi24", default)]
        pub toilet: Option<ToiletAvailabilityType>,
        #[yaserde(rename = "loading", prefix = "nsi24", default)]
        pub loading: Option<LoadingValue>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ArrayOfCoaches",
        namespace = "nsi24: http://thalesgroup.com/RTTI/2017-10-01/ldb/types",
        prefix = "nsi24",
    )]
    pub struct ArrayOfCoaches {
        #[yaserde(rename = "coach", prefix = "nsi24", default)]
        pub coach: Vec<CoachData>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "UncertaintyStatus",
        namespace = "nsi3: http://thalesgroup.com/RTTI/2021-11-01/ldb/types",
        prefix = "nsi3",
    )]
    pub struct UncertaintyStatus {
        #[yaserde(text, default)]
        pub body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "LoadingCategory",
        namespace = "nsi3: http://thalesgroup.com/RTTI/2021-11-01/ldb/types",
        prefix = "nsi3",
    )]
    pub struct LoadingCategory {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "UncertaintyType",
        namespace = "nsi3: http://thalesgroup.com/RTTI/2021-11-01/ldb/types",
        prefix = "nsi3",
    )]
    pub struct UncertaintyType {
        #[yaserde(rename = "status", attribute)]
        pub status: UncertaintyStatus,
        #[yaserde(rename = "reason", prefix = "nsi3", default)]
        pub reason: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetBoardRequestParams",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct GetBoardRequestParams {
        #[yaserde(rename = "numRows", prefix = "tns", default)]
        pub num_rows: u16,
        #[yaserde(rename = "crs", prefix = "tns", default)]
        pub crs: Crstype,
        #[yaserde(rename = "filterCrs", prefix = "tns", default)]
        pub filter_crs: Option<Crstype>,
        #[yaserde(rename = "filterType", prefix = "tns", default)]
        pub filter_type: Option<FilterType>,
        #[yaserde(rename = "timeOffset", prefix = "tns", default)]
        pub time_offset: Option<i32>,
        #[yaserde(rename = "timeWindow", prefix = "tns", default)]
        pub time_window: Option<i32>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetServiceDetailsRequestParams",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct GetServiceDetailsRequestParams {
        #[yaserde(rename = "serviceID", prefix = "tns", default)]
        pub service_id: ServiceIDType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "filterList",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct FilterList {
        #[yaserde(rename = "crs", prefix = "tns", default)]
        pub crs: Vec<Crstype>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetDeparturesRequestParams",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct GetDeparturesRequestParams {
        #[yaserde(rename = "crs", prefix = "tns", default)]
        pub crs: Crstype,
        #[yaserde(rename = "filterList", prefix = "tns", default)]
        pub filter_list: FilterList,
        #[yaserde(rename = "timeOffset", prefix = "tns", default)]
        pub time_offset: Option<i32>,
        #[yaserde(rename = "timeWindow", prefix = "tns", default)]
        pub time_window: Option<i32>,
    }

    pub type GetDepartureBoardRequest = GetBoardRequestParams;

    pub type GetArrivalBoardRequest = GetBoardRequestParams;

    pub type GetArrivalDepartureBoardRequest = GetBoardRequestParams;

    pub type GetServiceDetailsRequest = GetServiceDetailsRequestParams;

    pub type GetDepBoardWithDetailsRequest = GetBoardRequestParams;

    pub type GetArrBoardWithDetailsRequest = GetBoardRequestParams;

    pub type GetArrDepBoardWithDetailsRequest = GetBoardRequestParams;

    pub type GetNextDeparturesRequest = GetDeparturesRequestParams;

    pub type GetNextDeparturesWithDetailsRequest = GetDeparturesRequestParams;

    pub type GetFastestDeparturesRequest = GetDeparturesRequestParams;

    pub type GetFastestDeparturesWithDetailsRequest = GetDeparturesRequestParams;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "StationBoardResponseType",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct StationBoardResponseType {
        #[yaserde(rename = "GetStationBoardResult", prefix = "tns", default)]
        pub get_station_board_result: Option<StationBoard>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "StationBoardWithDetailsResponseType",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct StationBoardWithDetailsResponseType {
        #[yaserde(rename = "GetStationBoardResult", prefix = "tns", default)]
        pub get_station_board_result: Option<StationBoardWithDetails>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceDetailsResponseType",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct ServiceDetailsResponseType {
        #[yaserde(rename = "GetServiceDetailsResult", prefix = "tns", default)]
        pub get_service_details_result: Option<ServiceDetails>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DeparturesBoardResponseType",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct DeparturesBoardResponseType {
        #[yaserde(rename = "DeparturesBoard", prefix = "tns", default)]
        pub departures_board: Option<DeparturesBoard>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DeparturesBoardWithDetailsResponseType",
        namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/",
        prefix = "tns",
    )]
    pub struct DeparturesBoardWithDetailsResponseType {
        #[yaserde(rename = "DeparturesBoard", prefix = "tns", default)]
        pub departures_board: Option<DeparturesBoardWithDetails>,
    }

    pub type GetDepartureBoardResponse = StationBoardResponseType;

    pub type GetArrivalBoardResponse = StationBoardResponseType;

    pub type GetArrivalDepartureBoardResponse = StationBoardResponseType;

    pub type GetServiceDetailsResponse = ServiceDetailsResponseType;

    pub type GetDepBoardWithDetailsResponse = StationBoardWithDetailsResponseType;

    pub type GetArrBoardWithDetailsResponse = StationBoardWithDetailsResponseType;

    pub type GetArrDepBoardWithDetailsResponse = StationBoardWithDetailsResponseType;

    pub type GetNextDeparturesResponse = DeparturesBoardResponseType;

    pub type GetNextDeparturesWithDetailsResponse = DeparturesBoardWithDetailsResponseType;

    pub type GetFastestDeparturesResponse = DeparturesBoardResponseType;

    pub type GetFastestDeparturesWithDetailsResponse = DeparturesBoardWithDetailsResponseType;
}

pub mod ports {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    pub type GetDepartureBoardSoapIn = messages::GetDepartureBoardSoapIn;

    pub type GetDepartureBoardSoapOut = messages::GetDepartureBoardSoapOut;

    pub type GetArrivalBoardSoapIn = messages::GetArrivalBoardSoapIn;

    pub type GetArrivalBoardSoapOut = messages::GetArrivalBoardSoapOut;

    pub type GetArrivalDepartureBoardSoapIn = messages::GetArrivalDepartureBoardSoapIn;

    pub type GetArrivalDepartureBoardSoapOut = messages::GetArrivalDepartureBoardSoapOut;

    pub type GetServiceDetailsSoapIn = messages::GetServiceDetailsSoapIn;

    pub type GetServiceDetailsSoapOut = messages::GetServiceDetailsSoapOut;

    pub type GetDepBoardWithDetailsSoapIn = messages::GetDepBoardWithDetailsSoapIn;

    pub type GetDepBoardWithDetailsSoapOut = messages::GetDepBoardWithDetailsSoapOut;

    pub type GetArrBoardWithDetailsSoapIn = messages::GetArrBoardWithDetailsSoapIn;

    pub type GetArrBoardWithDetailsSoapOut = messages::GetArrBoardWithDetailsSoapOut;

    pub type GetArrDepBoardWithDetailsSoapIn = messages::GetArrDepBoardWithDetailsSoapIn;

    pub type GetArrDepBoardWithDetailsSoapOut = messages::GetArrDepBoardWithDetailsSoapOut;

    pub type GetNextDeparturesSoapIn = messages::GetNextDeparturesSoapIn;

    pub type GetNextDeparturesSoapOut = messages::GetNextDeparturesSoapOut;

    pub type GetNextDeparturesWithDetailsSoapIn = messages::GetNextDeparturesWithDetailsSoapIn;

    pub type GetNextDeparturesWithDetailsSoapOut = messages::GetNextDeparturesWithDetailsSoapOut;

    pub type GetFastestDeparturesSoapIn = messages::GetFastestDeparturesSoapIn;

    pub type GetFastestDeparturesSoapOut = messages::GetFastestDeparturesSoapOut;

    pub type GetFastestDeparturesWithDetailsSoapIn = messages::GetFastestDeparturesWithDetailsSoapIn;

    pub type GetFastestDeparturesWithDetailsSoapOut = messages::GetFastestDeparturesWithDetailsSoapOut;

    #[async_trait]
    pub trait LdbserviceSoap {
        async fn get_departure_board(&self, get_departure_board_soap_in: GetDepartureBoardSoapIn) -> Result<GetDepartureBoardSoapOut, Option<SoapFault>>;
        async fn get_arrival_board(&self, get_arrival_board_soap_in: GetArrivalBoardSoapIn) -> Result<GetArrivalBoardSoapOut, Option<SoapFault>>;
        async fn get_arrival_departure_board(&self, get_arrival_departure_board_soap_in: GetArrivalDepartureBoardSoapIn) -> Result<GetArrivalDepartureBoardSoapOut, Option<SoapFault>>;
        async fn get_service_details(&self, get_service_details_soap_in: GetServiceDetailsSoapIn) -> Result<GetServiceDetailsSoapOut, Option<SoapFault>>;
        async fn get_dep_board_with_details(&self, get_dep_board_with_details_soap_in: GetDepBoardWithDetailsSoapIn) -> Result<GetDepBoardWithDetailsSoapOut, Option<SoapFault>>;
        async fn get_arr_board_with_details(&self, get_arr_board_with_details_soap_in: GetArrBoardWithDetailsSoapIn) -> Result<GetArrBoardWithDetailsSoapOut, Option<SoapFault>>;
        async fn get_arr_dep_board_with_details(&self, get_arr_dep_board_with_details_soap_in: GetArrDepBoardWithDetailsSoapIn) -> Result<GetArrDepBoardWithDetailsSoapOut, Option<SoapFault>>;
        async fn get_next_departures(&self, get_next_departures_soap_in: GetNextDeparturesSoapIn) -> Result<GetNextDeparturesSoapOut, Option<SoapFault>>;
        async fn get_next_departures_with_details(&self, get_next_departures_with_details_soap_in: GetNextDeparturesWithDetailsSoapIn) -> Result<GetNextDeparturesWithDetailsSoapOut, Option<SoapFault>>;
        async fn get_fastest_departures(&self, get_fastest_departures_soap_in: GetFastestDeparturesSoapIn) -> Result<GetFastestDeparturesSoapOut, Option<SoapFault>>;
        async fn get_fastest_departures_with_details(&self, get_fastest_departures_with_details_soap_in: GetFastestDeparturesWithDetailsSoapIn) -> Result<GetFastestDeparturesWithDetailsSoapOut, Option<SoapFault>>;
    }
}

pub mod bindings {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    impl LdbserviceSoap {
        async fn send_soap_request<T: YaSerialize>(&self, request: &T, action: &str) -> SoapResponse {
            let body = to_string(request).expect("failed to generate xml");
            debug!("SOAP Request: {}", body);
            let mut req = self
                .client
                .post(&self.url)
                .body(body)
                .header("Content-Type", "text/xml")
                .header("Soapaction", action);
            trace!("SOAP Request: {:?}", req);
            let res = req.send().await?;
            let status = res.status();
            debug!("SOAP Status: {}", status);
            let txt = res.text().await.unwrap_or_default();
            debug!("SOAP Response: {}", txt);
            Ok((status, txt))
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetDepartureBoardSoapIn {
        #[yaserde(rename = "tns:GetDepartureBoard", default)]
        pub body: ports::GetDepartureBoardSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetDepartureBoardSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetDepartureBoardSoapIn,
    }

    impl GetDepartureBoardSoapInSoapEnvelope {
        #[must_use]
        pub fn new(access_token: AccessToken, body: SoapGetDepartureBoardSoapIn) -> Self {
            GetDepartureBoardSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: Some(Header { access_token }),
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetDepartureBoardSoapOut {
        #[yaserde(rename = "GetDepartureBoardResponse", default)]
        pub body: Option<ports::GetDepartureBoardSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetDepartureBoardSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetDepartureBoardSoapOut,
    }

    impl GetDepartureBoardSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetDepartureBoardSoapOut) -> Self {
            GetDepartureBoardSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrivalBoardSoapIn {
        #[yaserde(rename = "tns:GetArrivalBoard", default)]
        pub body: ports::GetArrivalBoardSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrivalBoardSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrivalBoardSoapIn,
    }

    impl GetArrivalBoardSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrivalBoardSoapIn) -> Self {
            GetArrivalBoardSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrivalBoardSoapOut {
        #[yaserde(rename = "GetArrivalBoardResponse", default)]
        pub body: Option<ports::GetArrivalBoardSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrivalBoardSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrivalBoardSoapOut,
    }

    impl GetArrivalBoardSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrivalBoardSoapOut) -> Self {
            GetArrivalBoardSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrivalDepartureBoardSoapIn {
        #[yaserde(rename = "tns:GetArrivalDepartureBoard", default)]
        pub body: ports::GetArrivalDepartureBoardSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrivalDepartureBoardSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrivalDepartureBoardSoapIn,
    }

    impl GetArrivalDepartureBoardSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrivalDepartureBoardSoapIn) -> Self {
            GetArrivalDepartureBoardSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrivalDepartureBoardSoapOut {
        #[yaserde(rename = "GetArrivalDepartureBoardResponse", default)]
        pub body: Option<ports::GetArrivalDepartureBoardSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrivalDepartureBoardSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrivalDepartureBoardSoapOut,
    }

    impl GetArrivalDepartureBoardSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrivalDepartureBoardSoapOut) -> Self {
            GetArrivalDepartureBoardSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetServiceDetailsSoapIn {
        #[yaserde(rename = "tns:GetServiceDetails", default)]
        pub body: ports::GetServiceDetailsSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetServiceDetailsSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetServiceDetailsSoapIn,
    }

    impl GetServiceDetailsSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetServiceDetailsSoapIn) -> Self {
            GetServiceDetailsSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetServiceDetailsSoapOut {
        #[yaserde(rename = "GetServiceDetailsResponse", default)]
        pub body: Option<ports::GetServiceDetailsSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetServiceDetailsSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetServiceDetailsSoapOut,
    }

    impl GetServiceDetailsSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetServiceDetailsSoapOut) -> Self {
            GetServiceDetailsSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetDepBoardWithDetailsSoapIn {
        #[yaserde(rename = "tns:GetDepBoardWithDetails", default)]
        pub body: ports::GetDepBoardWithDetailsSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetDepBoardWithDetailsSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetDepBoardWithDetailsSoapIn,
    }

    impl GetDepBoardWithDetailsSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetDepBoardWithDetailsSoapIn) -> Self {
            GetDepBoardWithDetailsSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetDepBoardWithDetailsSoapOut {
        #[yaserde(rename = "GetDepBoardWithDetailsResponse", default)]
        pub body: Option<ports::GetDepBoardWithDetailsSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetDepBoardWithDetailsSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetDepBoardWithDetailsSoapOut,
    }

    impl GetDepBoardWithDetailsSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetDepBoardWithDetailsSoapOut) -> Self {
            GetDepBoardWithDetailsSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrBoardWithDetailsSoapIn {
        #[yaserde(rename = "tns:GetArrBoardWithDetails", default)]
        pub body: ports::GetArrBoardWithDetailsSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrBoardWithDetailsSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrBoardWithDetailsSoapIn,
    }

    impl GetArrBoardWithDetailsSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrBoardWithDetailsSoapIn) -> Self {
            GetArrBoardWithDetailsSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrBoardWithDetailsSoapOut {
        #[yaserde(rename = "GetArrBoardWithDetailsResponse", default)]
        pub body: Option<ports::GetArrBoardWithDetailsSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrBoardWithDetailsSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrBoardWithDetailsSoapOut,
    }

    impl GetArrBoardWithDetailsSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrBoardWithDetailsSoapOut) -> Self {
            GetArrBoardWithDetailsSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrDepBoardWithDetailsSoapIn {
        #[yaserde(rename = "tns:GetArrDepBoardWithDetails", default)]
        pub body: ports::GetArrDepBoardWithDetailsSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrDepBoardWithDetailsSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrDepBoardWithDetailsSoapIn,
    }

    impl GetArrDepBoardWithDetailsSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrDepBoardWithDetailsSoapIn) -> Self {
            GetArrDepBoardWithDetailsSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetArrDepBoardWithDetailsSoapOut {
        #[yaserde(rename = "GetArrDepBoardWithDetailsResponse", default)]
        pub body: Option<ports::GetArrDepBoardWithDetailsSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetArrDepBoardWithDetailsSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetArrDepBoardWithDetailsSoapOut,
    }

    impl GetArrDepBoardWithDetailsSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetArrDepBoardWithDetailsSoapOut) -> Self {
            GetArrDepBoardWithDetailsSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetNextDeparturesSoapIn {
        #[yaserde(rename = "tns:GetNextDepartures", default)]
        pub body: ports::GetNextDeparturesSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetNextDeparturesSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetNextDeparturesSoapIn,
    }

    impl GetNextDeparturesSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetNextDeparturesSoapIn) -> Self {
            GetNextDeparturesSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetNextDeparturesSoapOut {
        #[yaserde(rename = "GetNextDeparturesResponse", default)]
        pub body: Option<ports::GetNextDeparturesSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetNextDeparturesSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetNextDeparturesSoapOut,
    }

    impl GetNextDeparturesSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetNextDeparturesSoapOut) -> Self {
            GetNextDeparturesSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetNextDeparturesWithDetailsSoapIn {
        #[yaserde(rename = "tns:GetNextDeparturesWithDetails", default)]
        pub body: ports::GetNextDeparturesWithDetailsSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetNextDeparturesWithDetailsSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetNextDeparturesWithDetailsSoapIn,
    }

    impl GetNextDeparturesWithDetailsSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetNextDeparturesWithDetailsSoapIn) -> Self {
            GetNextDeparturesWithDetailsSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetNextDeparturesWithDetailsSoapOut {
        #[yaserde(rename = "GetNextDeparturesWithDetailsResponse", default)]
        pub body: Option<ports::GetNextDeparturesWithDetailsSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetNextDeparturesWithDetailsSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetNextDeparturesWithDetailsSoapOut,
    }

    impl GetNextDeparturesWithDetailsSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetNextDeparturesWithDetailsSoapOut) -> Self {
            GetNextDeparturesWithDetailsSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetFastestDeparturesSoapIn {
        #[yaserde(rename = "tns:GetFastestDepartures", default)]
        pub body: ports::GetFastestDeparturesSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetFastestDeparturesSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetFastestDeparturesSoapIn,
    }

    impl GetFastestDeparturesSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetFastestDeparturesSoapIn) -> Self {
            GetFastestDeparturesSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetFastestDeparturesSoapOut {
        #[yaserde(rename = "GetFastestDeparturesResponse", default)]
        pub body: Option<ports::GetFastestDeparturesSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetFastestDeparturesSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetFastestDeparturesSoapOut,
    }

    impl GetFastestDeparturesSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetFastestDeparturesSoapOut) -> Self {
            GetFastestDeparturesSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetFastestDeparturesWithDetailsSoapIn {
        #[yaserde(rename = "tns:GetFastestDeparturesWithDetails", default)]
        pub body: ports::GetFastestDeparturesWithDetailsSoapIn,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetFastestDeparturesWithDetailsSoapInSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetFastestDeparturesWithDetailsSoapIn,
    }

    impl GetFastestDeparturesWithDetailsSoapInSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetFastestDeparturesWithDetailsSoapIn) -> Self {
            GetFastestDeparturesWithDetailsSoapInSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetFastestDeparturesWithDetailsSoapOut {
        #[yaserde(rename = "GetFastestDeparturesWithDetailsResponse", default)]
        pub body: Option<ports::GetFastestDeparturesWithDetailsSoapOut>,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soapenv"
    )]
    pub struct GetFastestDeparturesWithDetailsSoapOutSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: Option<String>,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetFastestDeparturesWithDetailsSoapOut,
    }

    impl GetFastestDeparturesWithDetailsSoapOutSoapEnvelope {
        #[must_use]
        pub fn new(body: SoapGetFastestDeparturesWithDetailsSoapOut) -> Self {
            GetFastestDeparturesWithDetailsSoapOutSoapEnvelope {
                encoding_style: Some(SOAP_ENCODING.to_string()),
                tnsattr: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    impl Default for LdbserviceSoap {
        fn default() -> Self {
            LdbserviceSoap {
                client: reqwest::Client::new(),
                url: "http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string(),
                credentials: None,
            }
        }
    }

    impl LdbserviceSoap {
        #[must_use]
        pub fn new(url: &str, credentials: Option<String>) -> Self {
            LdbserviceSoap {
                client: reqwest::Client::new(),
                url: url.to_string(),
                credentials,
            }
        }
    }

    pub struct LdbserviceSoap {
        client: reqwest::Client,
        url: String,
        credentials: Option<String>,
    }

    #[async_trait]
    impl ports::LdbserviceSoap for LdbserviceSoap {
        async fn get_departure_board(&self, get_departure_board_soap_in: ports::GetDepartureBoardSoapIn) -> Result<ports::GetDepartureBoardSoapOut, Option<SoapFault>> {
            let __request = GetDepartureBoardSoapInSoapEnvelope::new(
                AccessToken { token_value: self.credentials.clone().unwrap_or(String::default()) },
                SoapGetDepartureBoardSoapIn {
                body: get_departure_board_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetDepartureBoard")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetDepartureBoardSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arrival_board(&self, get_arrival_board_soap_in: ports::GetArrivalBoardSoapIn) -> Result<ports::GetArrivalBoardSoapOut, Option<SoapFault>> {
            let __request = GetArrivalBoardSoapInSoapEnvelope::new(SoapGetArrivalBoardSoapIn {
                body: get_arrival_board_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetArrivalBoard")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrivalBoardSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arrival_departure_board(&self, get_arrival_departure_board_soap_in: ports::GetArrivalDepartureBoardSoapIn) -> Result<ports::GetArrivalDepartureBoardSoapOut, Option<SoapFault>> {
            let __request = GetArrivalDepartureBoardSoapInSoapEnvelope::new(SoapGetArrivalDepartureBoardSoapIn {
                body: get_arrival_departure_board_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetArrivalDepartureBoard")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrivalDepartureBoardSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_service_details(&self, get_service_details_soap_in: ports::GetServiceDetailsSoapIn) -> Result<ports::GetServiceDetailsSoapOut, Option<SoapFault>> {
            let __request = GetServiceDetailsSoapInSoapEnvelope::new(SoapGetServiceDetailsSoapIn {
                body: get_service_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetServiceDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetServiceDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_dep_board_with_details(&self, get_dep_board_with_details_soap_in: ports::GetDepBoardWithDetailsSoapIn) -> Result<ports::GetDepBoardWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetDepBoardWithDetailsSoapInSoapEnvelope::new(SoapGetDepBoardWithDetailsSoapIn {
                body: get_dep_board_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetDepBoardWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetDepBoardWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arr_board_with_details(&self, get_arr_board_with_details_soap_in: ports::GetArrBoardWithDetailsSoapIn) -> Result<ports::GetArrBoardWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetArrBoardWithDetailsSoapInSoapEnvelope::new(SoapGetArrBoardWithDetailsSoapIn {
                body: get_arr_board_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetArrBoardWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrBoardWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arr_dep_board_with_details(&self, get_arr_dep_board_with_details_soap_in: ports::GetArrDepBoardWithDetailsSoapIn) -> Result<ports::GetArrDepBoardWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetArrDepBoardWithDetailsSoapInSoapEnvelope::new(SoapGetArrDepBoardWithDetailsSoapIn {
                body: get_arr_dep_board_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetArrDepBoardWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrDepBoardWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_next_departures(&self, get_next_departures_soap_in: ports::GetNextDeparturesSoapIn) -> Result<ports::GetNextDeparturesSoapOut, Option<SoapFault>> {
            let __request = GetNextDeparturesSoapInSoapEnvelope::new(SoapGetNextDeparturesSoapIn {
                body: get_next_departures_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetNextDepartures")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetNextDeparturesSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_next_departures_with_details(&self, get_next_departures_with_details_soap_in: ports::GetNextDeparturesWithDetailsSoapIn) -> Result<ports::GetNextDeparturesWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetNextDeparturesWithDetailsSoapInSoapEnvelope::new(SoapGetNextDeparturesWithDetailsSoapIn {
                body: get_next_departures_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetNextDeparturesWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetNextDeparturesWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_fastest_departures(&self, get_fastest_departures_soap_in: ports::GetFastestDeparturesSoapIn) -> Result<ports::GetFastestDeparturesSoapOut, Option<SoapFault>> {
            let __request = GetFastestDeparturesSoapInSoapEnvelope::new(SoapGetFastestDeparturesSoapIn {
                body: get_fastest_departures_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetFastestDepartures")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetFastestDeparturesSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_fastest_departures_with_details(&self, get_fastest_departures_with_details_soap_in: ports::GetFastestDeparturesWithDetailsSoapIn) -> Result<ports::GetFastestDeparturesWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetFastestDeparturesWithDetailsSoapInSoapEnvelope::new(SoapGetFastestDeparturesWithDetailsSoapIn {
                body: get_fastest_departures_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetFastestDeparturesWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetFastestDeparturesWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
    }

    impl LdbserviceSoap12 {
        async fn send_soap_request<T: YaSerialize>(&self, request: &T, action: &str) -> SoapResponse {
            let body = to_string(request).expect("failed to generate xml");
            debug!("SOAP Request: {}", body);
            let mut req = self
                .client
                .post(&self.url)
                .body(body)
                .header("Content-Type", "text/xml")
                .header("Soapaction", action);
            trace!("SOAP Request: {:?}", req);
            let res = req.send().await?;
            let status = res.status();
            debug!("SOAP Status: {}", status);
            let txt = res.text().await.unwrap_or_default();
            debug!("SOAP Response: {}", txt);
            Ok((status, txt))
        }
    }

    impl Default for LdbserviceSoap12 {
        fn default() -> Self {
            LdbserviceSoap12 {
                client: reqwest::Client::new(),
                url: "http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string(),
                credentials: None,
            }
        }
    }

    impl LdbserviceSoap12 {
        #[must_use]
        pub fn new(url: &str, credentials: Option<String>) -> Self {
            LdbserviceSoap12 {
                client: reqwest::Client::new(),
                url: url.to_string(),
                credentials,
            }
        }
    }

    pub struct LdbserviceSoap12 {
        client: reqwest::Client,
        url: String,
        credentials: Option<String>,
    }

    #[async_trait]
    impl ports::LdbserviceSoap for LdbserviceSoap12 {
        async fn get_departure_board(&self, get_departure_board_soap_in: ports::GetDepartureBoardSoapIn) -> Result<ports::GetDepartureBoardSoapOut, Option<SoapFault>> {
            let __request = GetDepartureBoardSoapInSoapEnvelope::new(
                AccessToken { token_value: self.credentials.clone().unwrap_or(String::default()) },
                SoapGetDepartureBoardSoapIn {
                    body: get_departure_board_soap_in,
                    xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
                });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetDepartureBoard")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetDepartureBoardSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arrival_board(&self, get_arrival_board_soap_in: ports::GetArrivalBoardSoapIn) -> Result<ports::GetArrivalBoardSoapOut, Option<SoapFault>> {
            let __request = GetArrivalBoardSoapInSoapEnvelope::new(SoapGetArrivalBoardSoapIn {
                body: get_arrival_board_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetArrivalBoard")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrivalBoardSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arrival_departure_board(&self, get_arrival_departure_board_soap_in: ports::GetArrivalDepartureBoardSoapIn) -> Result<ports::GetArrivalDepartureBoardSoapOut, Option<SoapFault>> {
            let __request = GetArrivalDepartureBoardSoapInSoapEnvelope::new(SoapGetArrivalDepartureBoardSoapIn {
                body: get_arrival_departure_board_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetArrivalDepartureBoard")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrivalDepartureBoardSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_service_details(&self, get_service_details_soap_in: ports::GetServiceDetailsSoapIn) -> Result<ports::GetServiceDetailsSoapOut, Option<SoapFault>> {
            let __request = GetServiceDetailsSoapInSoapEnvelope::new(SoapGetServiceDetailsSoapIn {
                body: get_service_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetServiceDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetServiceDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_dep_board_with_details(&self, get_dep_board_with_details_soap_in: ports::GetDepBoardWithDetailsSoapIn) -> Result<ports::GetDepBoardWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetDepBoardWithDetailsSoapInSoapEnvelope::new(SoapGetDepBoardWithDetailsSoapIn {
                body: get_dep_board_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetDepBoardWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetDepBoardWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arr_board_with_details(&self, get_arr_board_with_details_soap_in: ports::GetArrBoardWithDetailsSoapIn) -> Result<ports::GetArrBoardWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetArrBoardWithDetailsSoapInSoapEnvelope::new(SoapGetArrBoardWithDetailsSoapIn {
                body: get_arr_board_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetArrBoardWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrBoardWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_arr_dep_board_with_details(&self, get_arr_dep_board_with_details_soap_in: ports::GetArrDepBoardWithDetailsSoapIn) -> Result<ports::GetArrDepBoardWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetArrDepBoardWithDetailsSoapInSoapEnvelope::new(SoapGetArrDepBoardWithDetailsSoapIn {
                body: get_arr_dep_board_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetArrDepBoardWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetArrDepBoardWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_next_departures(&self, get_next_departures_soap_in: ports::GetNextDeparturesSoapIn) -> Result<ports::GetNextDeparturesSoapOut, Option<SoapFault>> {
            let __request = GetNextDeparturesSoapInSoapEnvelope::new(SoapGetNextDeparturesSoapIn {
                body: get_next_departures_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetNextDepartures")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetNextDeparturesSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_next_departures_with_details(&self, get_next_departures_with_details_soap_in: ports::GetNextDeparturesWithDetailsSoapIn) -> Result<ports::GetNextDeparturesWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetNextDeparturesWithDetailsSoapInSoapEnvelope::new(SoapGetNextDeparturesWithDetailsSoapIn {
                body: get_next_departures_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetNextDeparturesWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetNextDeparturesWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_fastest_departures(&self, get_fastest_departures_soap_in: ports::GetFastestDeparturesSoapIn) -> Result<ports::GetFastestDeparturesSoapOut, Option<SoapFault>> {
            let __request = GetFastestDeparturesSoapInSoapEnvelope::new(SoapGetFastestDeparturesSoapIn {
                body: get_fastest_departures_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetFastestDepartures")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetFastestDeparturesSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_fastest_departures_with_details(&self, get_fastest_departures_with_details_soap_in: ports::GetFastestDeparturesWithDetailsSoapIn) -> Result<ports::GetFastestDeparturesWithDetailsSoapOut, Option<SoapFault>> {
            let __request = GetFastestDeparturesWithDetailsSoapInSoapEnvelope::new(SoapGetFastestDeparturesWithDetailsSoapIn {
                body: get_fastest_departures_with_details_soap_in,
                xmlns: Some("http://thalesgroup.com/RTTI/2021-11-01/ldb/".to_string()),
            });

            let (status, response) = self.send_soap_request(&__request, "http://thalesgroup.com/RTTI/2015-05-14/ldb/GetFastestDeparturesWithDetails")
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetFastestDeparturesWithDetailsSoapOutSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body.expect("missing body"))
            } else {
                Err(r.body.fault)
            }
        }
    }
}

pub mod services {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;
}

pub mod multiref {
    //! This module contains the `MultiRef` type which is a wrapper around `Rc<RefCell<T>>` that implements `YaDeserialize` and `YaSerialize` for `T` and allows for multiple references to the same object.
    //! Inspired by [this](https://github.com/media-io/yaserde/issues/165#issuecomment-1810243674) comment on the yaserde repository.
    //! Needs `xml-rs` and `yaserde` as dependencies.

    use std::{cell::RefCell, ops::Deref, rc::Rc};
    use yaserde::{YaDeserialize, YaSerialize};

    pub struct MultiRef<T> {
        inner: Rc<RefCell<T>>,
    }

    impl<T: YaDeserialize + YaSerialize> YaDeserialize for MultiRef<T> {
        fn deserialize<R: std::io::prelude::Read>(
            reader: &mut yaserde::de::Deserializer<R>,
        ) -> Result<Self, String> {
            let inner = T::deserialize(reader)?;
            Ok(Self {
                inner: Rc::new(RefCell::new(inner)),
            })
        }
    }

    impl<T: YaDeserialize + YaSerialize> YaSerialize for MultiRef<T> {
        fn serialize<W: std::io::prelude::Write>(
            &self,
            writer: &mut yaserde::ser::Serializer<W>,
        ) -> Result<(), String> {
            self.inner.as_ref().borrow().serialize(writer)?;
            Ok(())
        }

        fn serialize_attributes(
            &self,
            attributes: Vec<xml::attribute::OwnedAttribute>,
            namespace: xml::namespace::Namespace,
        ) -> Result<
            (
                Vec<xml::attribute::OwnedAttribute>,
                xml::namespace::Namespace,
            ),
            String,
        > {
            self.inner
                .as_ref()
                .borrow()
                .serialize_attributes(attributes, namespace)
        }
    }

    impl<T: YaDeserialize + YaSerialize + Default> Default for MultiRef<T> {
        fn default() -> Self {
            Self {
                inner: Rc::default(),
            }
        }
    }

    impl<T: YaDeserialize + YaSerialize> Clone for MultiRef<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }

    impl<T: YaDeserialize + YaSerialize + std::fmt::Debug> std::fmt::Debug for MultiRef<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.inner.as_ref().borrow().fmt(f)
        }
    }

    impl<T> Deref for MultiRef<T> {
        type Target = Rc<RefCell<T>>;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
}

