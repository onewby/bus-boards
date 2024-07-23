use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::Mutex;
use futures::StreamExt;

use geographiclib_rs::{Geodesic, InverseGeodesic};
use itertools::Itertools;
use memmap::Mmap;
use piz::read::{as_tree, FileTree};
use piz::ZipArchive;
use polars::export::rayon::iter::IntoParallelRefIterator;
use polars::export::rayon::iter::ParallelIterator;
use rusqlite::{Connection, params};
use rusqlite::Error::QueryReturnedNoRows;
use rusqlite::functions::FunctionFlags;
use serde::{Deserialize, Serialize};

use BusBoardsServer::download_if_old;
use crate::open_db;

pub fn map_flix_stops(db: &mut Connection, db_path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    println!("Mapping Flix stops");
    download_if_old("http://gtfs.gis.flix.tech/gtfs_generic_eu.zip", "gtfs/flix.zip")?;

    let zip_file = File::open("gtfs/flix.zip")?;
    let mapping = unsafe { Mmap::map(&zip_file)? };
    let archive = ZipArchive::new(&mapping)?;
    let dir = as_tree(archive.entries())?;

    let file = dir.lookup("stops.txt")?;
    let mut rdr = csv::Reader::from_reader(archive.read(file)?);

    let map = HashMap::new();
    let map_mutex = Mutex::new(map);
    db.execute("INSERT OR IGNORE INTO localities (code, name, qualifier, parent, lat, long) VALUES ('Europe', 'Europe', NULL, NULL, 50.0, 9.0)", [])?;

    let stops = rdr.deserialize().map(|r: Result<FlixGTFSStop, _>| r.unwrap()).collect_vec();
    stops.par_iter().for_each(|record| {
        let mut db = open_db(db_path).unwrap();
        add_distance_function(&mut db).expect("Error adding distance function");
        if let Ok((stop, stance, indicator)) = get_nearest_stop(&mut db, record.stop_lat, record.stop_lon, record.stop_name.as_str()) {
            let indicator = indicator.unwrap_or("".to_string());
            if indicator.eq_ignore_ascii_case("at") {
                // if this stop is called at - use this stop
                map_mutex.lock().unwrap().insert(record.stop_id.to_string(), stance);
            } else if let Ok(code) = db.query_row("SELECT code FROM stances WHERE stop=? AND lower(indicator)='at' LIMIT 1", [], |row| row.get::<_, String>(0)) {
                // if this stop contains any stop with at - use that
                map_mutex.lock().unwrap().insert(record.stop_id.to_string(), code);
            } else {
                // else create stance in this stop
                let mut stmt = db.prepare_cached("INSERT INTO stances (code, street, indicator, lat, long, stop, crs) VALUES (?, NULL, 'at', ?, ?, ?, NULL)").unwrap();
                stmt.execute(params![&record.stop_id, &record.stop_lat, &record.stop_lon, stop]).expect("Stance create fail w/ existing stop");
            }
        } else {
            // try to find existing stop
            let stop_id: f64 = db.query_row("SELECT id FROM stops WHERE name=? AND locality='Europe'", [&record.stop_name], |row| row.get(0))
                .unwrap_or_else(|e| {
                    // ...or create stop if this is not found
                    // try to find a suitable locality
                    let (locality_code, locality_name): (String, String) = db.query_row(
                        "SELECT localities.code, (SELECT GROUP_CONCAT(name, ' › ') FROM (
                                    WITH RECURSIVE
                                        find_parent_names(level, code) AS (
                                            VALUES(0, localities.code)
                                            UNION
                                            SELECT level+1, parent FROM localities, find_parent_names
                                            WHERE localities.code=find_parent_names.code
                                        )
                                    SELECT name FROM localities, find_parent_names
                                    WHERE localities.code = find_parent_names.code
                                    ORDER BY level desc
                            )), geo_distance(localities.lat, localities.long, ?, ?) as dist
                            FROM localities WHERE dist < 5000 ORDER BY dist LIMIT 1", 
                        [record.stop_lat, record.stop_lon], 
                        |row| Ok((row.get(0)?, row.get(1)?))
                    ).unwrap_or(("Europe".to_string(), "Europe".to_string()));
                    // attempt to remove a city name prefix
                    let loc_prefix = locality_name.split(" › ").next().unwrap_or(locality_name.as_str());
                    let stop_name = record.stop_name.strip_prefix(format!("{loc_prefix} ").as_str())
                        .unwrap_or(record.stop_name.as_str());
                    // then create the stop
                    let mut stmt = db.prepare_cached("INSERT INTO stops (name, locality, locality_name) VALUES (?, ?, ?) RETURNING id").unwrap();
                    stmt.query_row([stop_name, &locality_code, &locality_name], |row| row.get(0)).expect("Stop create fail")
                });
            if record.stop_name == "Manchester Airport" {
                println!("Inserting Manchester Airport into stop {}", stop_id);
            }
            // create stance
            let mut stmt = db.prepare_cached("INSERT INTO stances (code, street, indicator, lat, long, stop, crs) VALUES (?, NULL, 'at', ?, ?, ?, NULL)").unwrap();
            stmt.execute(params![&record.stop_id, &record.stop_lat, &record.stop_lon, stop_id]).expect("Stance create fail without existing stop");
        }
    });
    Ok(map_mutex.into_inner().unwrap())
}

// https://gist.github.com/graydon/11198540
const UK_LAT_LON: (f64, f64, f64, f64) = (-7.57216793459, 49.959999905, 1.68153079591, 58.6350001085);

fn get_nearest_stop(db: &mut Connection, lat: f64, lon: f64, stop_name: &str) -> rusqlite::Result<(u64, String, Option<String>)> {
    if lat <= UK_LAT_LON.1 || lat >= UK_LAT_LON.3 || lon <= UK_LAT_LON.0 || lon >= UK_LAT_LON.2 {
        return Err(QueryReturnedNoRows);
    }
    db.query_row("SELECT stop,code,indicator,min(geo_distance(lat, long, ?, ?)) as dist FROM stances GROUP BY stop HAVING dist < 50 ORDER BY dist LIMIT 1",
   [lat, lon], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
}

fn add_distance_function(db: &mut Connection) -> rusqlite::Result<()> {
    db.create_scalar_function("geo_distance", 4, FunctionFlags::SQLITE_DETERMINISTIC | FunctionFlags::SQLITE_INNOCUOUS, move |ctx| {
        Ok::<f64, rusqlite::Error>(Geodesic::wgs84().inverse(ctx.get(0)?, ctx.get(1)?, ctx.get(2)?, ctx.get(3)?))
    })
}

#[derive(Serialize, Deserialize)]
struct FlixGTFSStop {
    stop_id: String,
    stop_name: String,
    stop_lat: f64,
    stop_lon: f64,
    stop_timezone: String
}

struct StopResult {
    stop: String,
    distance: String
}