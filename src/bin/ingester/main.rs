mod sources;
mod gtfs;
mod cleanup;
mod traveline;

use std::fs;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::iter::Iterator;
use itertools::Itertools;
use piz::read::FileTree;
use rusqlite::Connection;
use rustls::client::ServerCertVerifier;
use serde::{Deserialize, Serialize};
use crate::gtfs::process_source;
use crate::sources::SOURCES;

const SQL_INDEXES: &str = include_str!("sql/indexes.sql");
const SQL_MODEL: &str = include_str!("sql/model.sql");
const GTFS_TABLES: [&str; 6] = ["stop_times", "trips", "calendar_dates", "calendar", "routes", "agency"];

fn main() {
    let mut connection = open_db().expect("DB init error");
    create_tables(&connection).expect("Table create error");
    clear_tables(&connection).expect("Table reconstruction error");
    for source in SOURCES {
        process_source(&source).expect("Download error");
    }
    create_indexes(&mut connection).expect("Index creation error");
    cleanup::cleanup(&mut connection).expect("Cleanup error");
    traveline::download_noc(&mut connection).expect("Traveline error");
    connection.close().expect("Could not close connection");
    println!("Done!")
}

fn open_db() -> Result<Connection, rusqlite::Error> {
    println!("Opening database");
    let conn = Connection::open("stops.sqlite")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

fn create_tables(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("Initialising tables");
    conn.execute_batch(SQL_MODEL)?;
    Ok(())
}

fn clear_tables(conn: &Connection) -> Result<(), Box<dyn Error>> {
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    for table in GTFS_TABLES {
        conn.execute(format!("DELETE FROM {table}").as_str(), [])?;
    }
    create_tables(conn)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

pub fn create_indexes(conn: &mut Connection) -> rusqlite::Result<()> {
    println!("Creating indexes");
    conn.execute_batch(SQL_INDEXES)
}

type Localities = HashMap<LocalityCode, HashMap<StopName, Vec<Stance>>>;
type LocalityCode = String;
type StopName = String;

#[derive(Serialize, Deserialize)]
struct Stance {
    #[serde(rename = "ATCOCode")]
    atco_code: String,
    #[serde(rename = "Lat")]
    lat: f64,
    #[serde(rename = "Long")]
    long: f64,
    #[serde(rename = "Street")]
    street: Option<String>,
    #[serde(rename = "Indicator")]
    indicator: Option<String>,
    #[serde(rename = "Arrival")]
    arrival: bool,
    #[serde(rename = "CrsRef")]
    crs: Option<String>
}

fn load_localities_json() -> Localities {
    let json_str = fs::read_to_string("localities.json").expect("Cannot find localities.json");
    serde_json::from_str(&json_str).expect("JSON parse fail")
}