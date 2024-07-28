use std::error::Error;
use std::fmt;
use log::{debug, info, trace, warn};
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use yaserde::{YaDeserialize, YaSerialize};
use yaserde::de::from_str;
use yaserde::ser::to_string;

pub struct LDBService {
    client: reqwest::Client,
    access_token: String
}

impl LDBService {
    pub fn new(access_token: String) -> LDBService {
        LDBService {
            client: reqwest::Client::builder().no_gzip().build().unwrap(),
            access_token,
        }
    }
    
    pub async fn get_departure_board(&self, request: GetDepartureBoardRequest) -> Result<GetDepartureBoardResponse, Option<SoapFault>> {
        self.generic_get::<_, GetDepartureBoardResponseWrapper>(GetDepartureBoardRequestWrapper { request }, "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetDepartureBoard").await
            .map(|w| w.response)
    }

    async fn generic_get<Req: YaSerialize + YaDeserialize, Resp: YaDeserialize + yaserde::YaSerialize + std::fmt::Debug>(&self, request: Req, action: &str) -> Result<Resp, Option<SoapFault>> {
        let send_envelope = DarwinEnvelope {
            header: Some(Header {
                access_token: AccessToken {
                    token_value: self.access_token.to_string(),
                },
            }),
            body: request,
            fault: None,
        };
        let (status, resp) = self.send_soap_request(&send_envelope, action)
            .await
            .map_err(|err| {
                warn!("Failed to send SOAP request: {:?}", err);
                None
            })?;;
        let r: DarwinEnvelope<Resp> = from_str(resp.as_str()).map_err(|err| {
            warn!("Failed to unmarshal SOAP response: {:?}", err);
            None
        })?;
        if status.is_success() {
            Ok(r.body)
        } else {
            if r.fault.is_some() {
                warn!("{}", r.fault.as_ref().unwrap());
            }
            Err(r.fault)
        }
    }
    
    async fn send_soap_request<T: YaSerialize>(&self, request: &T, action: &str) -> SoapResponse {
        let body = to_string(request).expect("failed to generate xml");
        debug!("SOAP Request: {}", body);
        let mut req = self
            .client
            .post("https://lite.realtime.nationalrail.co.uk/OpenLDBWS/ldb12.asmx")
            .body(body)
            .header("Content-Type", "text/xml")
            .header("SOAPAction", action);
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
#[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
)]
pub struct DarwinEnvelope<T: YaSerialize + yaserde::YaDeserialize> {
    #[yaserde(rename = "Header", prefix = "soap")]
    pub header: Option<Header>,
    #[yaserde(rename = "Body", prefix = "soap")]
    pub body: T,
    #[yaserde(rename = "Fault", default)]
    pub fault: Option<SoapFault>
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "Header", namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/", prefix="soap")]
pub struct Header {
    #[yaserde(rename = "AccessToken", prefix = "typ")]
    pub access_token: AccessToken,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
    rename = "AccessToken",
    namespace = "typ: http://thalesgroup.com/RTTI/2013-11-28/Token/types",
    namespace = "http://thalesgroup.com/RTTI/2013-11-28/Token/types",
    prefix = "typ")]
pub struct AccessToken {
    #[yaserde(rename = "TokenValue", prefix = "typ", default)]
    pub token_value: String
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "Fault", namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/", prefix="soap")]
pub struct SoapFault {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: Option<String>,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize)]
pub struct GetDepartureBoardRequestWrapper {
    #[yaserde(rename = "GetDepartureBoardRequest", prefix="tns")]
    request: GetDepartureBoardRequest
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "GetDepartureBoardRequest", namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/", prefix="tns")]
pub struct GetDepartureBoardRequest {
    #[yaserde(rename = "numRows", prefix = "tns", default)]
    pub num_rows: u16,
    #[yaserde(rename = "crs", prefix = "tns", default)]
    pub crs: String,
    #[yaserde(rename = "filterCrs", prefix = "tns", default)]
    pub filter_crs: Option<String>,
    #[yaserde(rename = "filterType", prefix = "tns", default)]
    pub filter_type: Option<String>,
    #[yaserde(rename = "timeOffset", prefix = "tns", default)]
    pub time_offset: Option<i32>,
    #[yaserde(rename = "timeWindow", prefix = "tns", default)]
    pub time_window: Option<i32>,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct GetDepartureBoardResponseWrapper {
    #[yaserde(rename = "GetDepartureBoardResponse", namespace = "http://thalesgroup.com/RTTI/2021-11-01/ldb/")]
    pub response: GetDepartureBoardResponse
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "GetDepartureBoardResponse", namespace = "tns: http://thalesgroup.com/RTTI/2021-11-01/ldb/", prefix="tns")]
pub struct GetDepartureBoardResponse {
    #[yaserde(rename = "GetStationBoardResult", prefix = "tns", default)]
    pub response: Option<StationBoard>
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "GetStationBoardResult",
    namespace = "lt: http://thalesgroup.com/RTTI/2012-01-13/ldb/types",
    namespace = "lt8: http://thalesgroup.com/RTTI/2021-11-01/ldb/types",
    namespace = "lt6: http://thalesgroup.com/RTTI/2017-02-03/ldb/types",
    namespace = "lt7: http://thalesgroup.com/RTTI/2017-10-01/ldb/types",
    namespace = "lt4: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
    namespace = "lt5: http://thalesgroup.com/RTTI/2016-02-16/ldb/types",
    namespace = "lt2: http://thalesgroup.com/RTTI/2014-02-20/ldb/types",
    namespace = "lt3: http://thalesgroup.com/RTTI/2015-05-14/ldb/types")]
pub struct StationBoard {
    #[yaserde(rename = "generatedAt", prefix = "lt4", default)]
    pub generated_at: String,
    #[yaserde(rename = "locationName", prefix = "lt4", default)]
    pub location_name: String,
    #[yaserde(rename = "crs", prefix = "lt4", default)]
    pub crs: String,
    #[yaserde(rename = "filterLocationName", prefix = "lt4")]
    pub filter_location_name: Option<String>,
    #[yaserde(rename = "filtercrs", prefix = "lt4")]
    pub filtercrs: Option<String>,
    #[yaserde(rename = "filterType", prefix = "lt4")]
    pub filter_type: Option<String>,
    #[yaserde(rename = "platformAvailable", prefix = "lt4")]
    pub platform_available: Option<bool>,
    #[yaserde(rename = "areServicesAvailable", prefix = "lt4")]
    pub are_services_available: Option<bool>,
    #[yaserde(rename = "trainServices", default = "default_services")]
    pub train_services: Services,
    #[yaserde(rename = "busServices", default = "default_services")]
    pub bus_services: Services,
    #[yaserde(rename = "ferryServices", default = "default_services")]
    pub ferry_services: Services,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone, PartialEq)]
#[yaserde(namespace = "lt8: http://thalesgroup.com/RTTI/2021-11-01/ldb/types", prefix = "lt8")]
pub struct Services {
    #[yaserde(default, flatten, rename = "service", prefix = "lt8")]
    pub services: Vec<Service>
}

impl Services {
    pub fn iter(&self) -> impl Iterator<Item=&Service> {
        self.services.iter()
    }
}

fn default_services() -> Services {
    Services::default()
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone, PartialEq)]
#[yaserde(rename = "service",
    namespace = "lt8: http://thalesgroup.com/RTTI/2021-11-01/ldb/types",
    namespace = "lt4: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
    prefix = "lt8"
)]
pub struct Service {
    #[yaserde(rename = "origin", default = "new_termini")]
    pub origin: Termini,
    #[yaserde(rename = "destination", default = "new_termini")]
    pub destination: Termini,
    #[yaserde(rename = "currentOrigins", default = "new_termini")]
    pub current_origins: Termini,
    #[yaserde(rename = "currentDestinations", default = "new_termini")]
    pub current_destinations: Termini,
    #[yaserde(rename = "sta", prefix = "lt4")]
    pub sta: Option<String>,
    #[yaserde(rename = "eta", prefix = "lt4")]
    pub eta: Option<String>,
    #[yaserde(rename = "std", prefix = "lt4")]
    pub std: Option<String>,
    #[yaserde(rename = "etd", prefix = "lt4")]
    pub etd: Option<String>,
    #[yaserde(rename = "platform", prefix = "lt4")]
    pub platform: Option<String>,
    #[yaserde(rename = "operator", prefix = "lt4", default)]
    pub operator: String,
    #[yaserde(rename = "operatorCode", prefix = "lt4", default)]
    pub operator_code: String,
    #[yaserde(rename = "isCircularRoute", prefix = "lt4", default)]
    pub is_circular_route: Option<bool>,
    #[yaserde(rename = "serviceID", prefix = "lt4", default)]
    pub service_id: String,
    #[yaserde(rename = "adhocAlerts", prefix = "lt4", default = "new_strings")]
    pub adhoc_alerts: Vec<String>,
}

pub fn new_termini() -> Termini {
    Termini::default()
}

pub fn new_strings() -> Vec<String> {
    Vec::new()
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone, PartialEq)]
#[yaserde(
    namespace = "lt5: http://thalesgroup.com/RTTI/2016-02-16/ldb/types",
    prefix = "lt5",
)]
pub struct Termini {
    #[yaserde(rename = "location", prefix = "lt4", default)]
    pub locations: Vec<ServiceLocation>
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone, PartialEq)]
#[yaserde(
    rename = "ServiceLocation",
    namespace = "lt4: http://thalesgroup.com/RTTI/2015-11-27/ldb/types",
    prefix = "lt4",
)]
pub struct ServiceLocation {
    #[yaserde(rename = "locationName", prefix = "lt4", default)]
    pub location_name: String,
    #[yaserde(rename = "crs", prefix = "lt4", default)]
    pub crs: String,
    #[yaserde(rename = "via", prefix = "lt4", default)]
    pub via: Option<String>,
    #[yaserde(rename = "futureChangeTo", prefix = "lt4", default)]
    pub future_change_to: Option<String>,
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