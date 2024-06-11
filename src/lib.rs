use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

pub mod config;

#[derive(Copy, Clone, Display, EnumIter)]
#[derive(Eq, Hash, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum GTFSResponder {
    BODS, DISRUPTIONS, EMBER, PASSENGER, LOTHIAN, STAGECOACH, COACHES, FIRST
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct RPCConfiguration {
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64
}
