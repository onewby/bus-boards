use config::{Config, Environment, File, Map};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::GTFSResponder;
use crate::RPCConfiguration;

pub type SourceURL = String;
pub type OperatorName = String;

#[derive(Serialize, Deserialize, Default)]
pub struct BBConfig {
    pub listeners: Vec<GTFSResponder>,
    pub update_interval_days: u8,
    pub passenger: Map<SourceURL, Map<OperatorName, PassengerSource>>,
    pub stagecoach: StagecoachConfig,
    pub coaches: CoachesConfig,
    pub first: FirstConfig,
    pub lothian: LothianConfig
}

impl BBConfig {
    pub fn is_enabled(&self, resp: GTFSResponder) -> bool {
        self.listeners.contains(&resp)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct PassengerSource {
    pub gtfs: String,
    pub op_code: String
}

#[derive(Serialize, Deserialize, Default)]
pub struct StagecoachConfig {
    pub regional_operators: Vec<String>,
    pub local_operators: Map<String, String>
}

#[derive(Serialize, Deserialize, Default)]
pub struct CoachesConfig {
    pub operators: Vec<String>,
    pub route_overrides: Map<String, String>
}

#[derive(Serialize, Deserialize, Default)]
pub struct LothianConfig {
    pub operators: Map<String, String>
}

#[derive(Serialize, Deserialize, Default)]
pub struct FirstConfig {
    pub api_key: String,
    pub operators: Map<String, String>,
    pub bounds: Map<String, RPCConfiguration>
}

pub fn load_config() -> BBConfig {
    let settings = Config::builder()
        // Add in config.toml and sources.yaml
        .add_source(File::with_name("config").required(false))
        .add_source(File::with_name("sources"))
        .add_source(File::with_name("private.config").required(false))
        // Add in settings from the environment (with a prefix of BUS)
        .add_source(Environment::with_prefix("BUS"))
        .set_default("listeners", GTFSResponder::iter().map(|r| r.to_string()).collect::<Vec<String>>()).unwrap()
        .build()
        .unwrap();
    settings.try_deserialize().unwrap_or_else(|err| {
        eprintln!("{}", err);
        BBConfig::default()
    })
}