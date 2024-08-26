use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use tempfile::NamedTempFile;

pub mod config;

#[derive(Copy, Clone, Display, EnumIter)]
#[derive(Eq, Hash, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum GTFSResponder {
    BODS, DISRUPTIONS, EMBER, PASSENGER, LOTHIAN, STAGECOACH, COACHES, FIRST, TFL
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct RPCConfiguration {
    pub min_lon: f64,
    pub max_lon: f64,
    pub min_lat: f64,
    pub max_lat: f64
}

pub fn download(url: &str, dst: &str) -> Result<File, Box<dyn Error>> {
    println!("Downloading {dst} from {url}");
    if Path::new(dst).exists() {
        fs::remove_file(dst)?;
    }
    let mut headers = HeaderMap::new();
    headers.append("accept", "*/*".parse().unwrap());
    let mut temp_file = NamedTempFile::new()?;
    reqwest::blocking::Client::builder()
        .timeout(None).build()?
        .get(url).headers(headers).send()?
        .copy_to(temp_file.as_file_mut())?;

    fs::rename(temp_file.path(), dst)?;
    Ok(File::open(dst)?)
}

pub fn download_if_old(url: &str, dst: &str) -> Result<File, Box<dyn Error>> {
    let path = Path::new(dst);
    if let Ok(md) = path.metadata() {
        if SystemTime::now().duration_since(md.modified()?)?.as_secs() < (24 * 60 * 60) {
            println!("- {dst} is still new - skipping.");
            return Ok(File::open(dst)?);
        } else {
            println!("- {dst} is old - redownloading.")
        }
    }
    download(url, dst)
}