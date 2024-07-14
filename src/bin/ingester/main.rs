mod sources;
mod gtfs;
mod cleanup;
mod traveline;
mod localities;
mod grouping;
mod locality_changes;

use std::error::Error;
use std::io::{Read, Write};
use std::iter::Iterator;
use std::string::ToString;
use std::fs::remove_file;
use itertools::Itertools;
use piz::read::FileTree;
use rusqlite::Connection;
use rustls::client::ServerCertVerifier;
use serde::{Deserialize, Serialize};
use crate::cleanup::cleanup;
use crate::grouping::group_stances;
use crate::gtfs::process_source;
use crate::localities::{insert_localities, insert_stops};
use crate::sources::SOURCES;
use crate::traveline::download_noc;

const DEFAULT_DB_PATH: &str = "stops.sqlite";
const SQL_INDEXES: &str = include_str!("sql/indexes.sql");
const SQL_MODEL: &str = include_str!("sql/model.sql");

fn main() {
    let db_path = std::env::var("BUSES_DB_PATH").unwrap_or(DEFAULT_DB_PATH.to_string());
    let _ = remove_file(db_path.as_str());
    let _ = remove_file(format!("{db_path}-shm"));
    let _ = remove_file(format!("{db_path}-wal"));

    group_stances().expect("Stance grouping error");

    let mut connection = open_db(db_path.as_str()).expect("DB init error");
    create_tables(&connection).expect("Table create error");
    insert_localities(&mut connection).expect("Locality insert error");
    insert_stops(&mut connection).expect("Stop insert error");
    for source in SOURCES {
        process_source(&mut connection, &source).expect("Download error");
    }
    create_indexes(&mut connection).expect("Index creation error");
    cleanup(&mut connection).expect("Cleanup error");
    download_noc(&mut connection).expect("Traveline error");
    connection.close().expect("Could not close connection");
    println!("Done!")
}

fn open_db(db_path: &str) -> Result<Connection, rusqlite::Error> {
    println!("Opening database");
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

fn create_tables(conn: &Connection) -> Result<(), Box<dyn Error>> {
    println!("Initialising tables");
    conn.execute_batch(SQL_MODEL)?;
    Ok(())
}

pub fn create_indexes(conn: &mut Connection) -> rusqlite::Result<()> {
    println!("Creating indexes");
    conn.execute_batch(SQL_INDEXES)
}