use std::{fs, iter};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use geo_types::Coord;
use itertools::Itertools;
use memmap::Mmap;
use phf::phf_map;
use piz::read::{as_tree, DirectoryContents, FileTree, ZipArchive};
use rusqlite::{Connection, params_from_iter};
use serde::Deserialize;

use BusBoardsServer::download_if_old;

use crate::sources::Source;

pub fn process_source(db: &mut Connection, source: &Source) -> Result<(), Box<dyn Error>> {
    // Download source
    fs::create_dir_all("gtfs")?;
    download_if_old(source.url, source.path)?;
    // Import zip
    import_zip(db, source)
}

fn import_zip(db: &mut Connection, source: &Source) -> Result<(), Box<dyn Error>> {
    let path = Path::new(source.path);
    let timer = Instant::now();
    let imports: Imports = [
        ("agency.txt", "REPLACE INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang) VALUES (?, ?, ?, ?, ?)",
         vec!["agency_id", "agency_name", "agency_url", "agency_timezone", "agency_lang"], phf_map! {}, false),
        ("routes.txt", "REPLACE INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type) VALUES (?, ?, ?, ?, ?)",
         vec!["route_id", "agency_id", "route_short_name", "route_long_name", "route_type"], phf_map! {"agency_id" => "Ember"}, false),
        ("calendar.txt", "REPLACE INTO calendar (service_id, start_date, end_date, validity) VALUES (?11||?1, cast(?2 as integer), ?3, (?4 + (?5 << 1) + (?6 << 2) + (?7 << 3) + (?8 << 4) + (?9 << 5) + (?10 << 6)))",
         vec!["service_id", "start_date", "end_date", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"], phf_map! {}, true),
        ("calendar_dates.txt", "REPLACE INTO calendar_dates (service_id, date, exception_type) VALUES (?4||?1, ?2, ?3)",
         vec!["service_id", "date", "exception_type"], phf_map! {}, true),
        ("trips.txt", "REPLACE INTO trips (route_id, service_id, trip_id, trip_headsign, shape_id) VALUES (?1, ?6||?2, ?6||?3, ?4, ?6||?5)",
         vec!["route_id", "service_id", "trip_id", "trip_headsign", "shape_id"], phf_map! {}, true),
        ("stop_times.txt", "REPLACE INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, timepoint, stop_headsign, pickup_type, drop_off_type) VALUES (?10||?1, substr(?2, 1, 2)*3600+substr(?2, 4, 2)*60+substr(?2, 7, 2), substr(?3, 1, 2)*3600+substr(?3, 4, 2)*60+substr(?3, 7, 2), ?4, ?5, ?6, NULLIF(?7, ''), ?8, ?9)",
         vec!["trip_id", "arrival_time", "departure_time", "stop_id", "stop_sequence", "timepoint", "stop_headsign", "pickup_type", "drop_off_type"], phf_map! {}, true)
    ];

    let zip_file = File::open(path)?;
    let mapping = unsafe { Mmap::map(&zip_file)? };
    let archive = ZipArchive::new(&mapping)?;
    let dir = as_tree(archive.entries())?;
    let file_name = path.file_name().unwrap().to_str().unwrap();

    for (subfile_name, stmt, indexes, defaults, add_prefix) in imports {
        println!("Importing {} for {}", subfile_name, file_name);
        import_txt_file(&archive, &dir, subfile_name, db, stmt, &indexes, &defaults, if add_prefix { Some(source.prefix) } else { None }).expect(subfile_name);
    }

    println!("Importing shapes.txt for {}", file_name);
    import_shapes(&archive, Some(source.prefix.to_string()), db)?;

    println!("{}s to import {}", timer.elapsed().as_secs(), file_name);

    Ok(())
}

fn import_txt_file(archive: &ZipArchive, dir: &DirectoryContents, file_name: &str, db: &mut Connection, stmt_str: &str, indexes: &Vec<&str>, defaults: &phf::Map<&str, &str>, prefix: Option<&str>) -> Result<(), Box<dyn Error>> {
    let file = dir.lookup(file_name)?;
    let stream_reader = BufReader::new(archive.read(file)?);
    let mut rdr = csv::Reader::from_reader(stream_reader);
    let headers = rdr.headers()?.clone();
    let headers: HashMap<_, _> = headers.iter().enumerate().map(|(i, hdr)| (hdr, i)).collect();
    let mut record = csv::ByteRecord::new();
    while !rdr.is_done() {
        let tx = db.transaction()?;
        {
            let mut stmt = tx.prepare(stmt_str)?;
            let mut i: u32 = 0;
            while i < 10000 && rdr.read_byte_record(&mut record)? {
                let optional_prefix_iter: Box<dyn Iterator<Item=Option<&str>>> = match prefix {
                    None => Box::new(iter::empty()),
                    Some(prefix) => Box::new(iter::once(Some(prefix)))
                };
                let params = indexes.iter()
                    .map(|hdr_name| {
                        let located_value = unsafe { std::str::from_utf8_unchecked(headers.get(hdr_name).and_then(|i| record.get(*i)).unwrap_or(EMPTY_SLICE)) };
                        if located_value.is_empty() {
                            defaults.get(hdr_name).map(|str| *str)
                        } else {
                            Some(located_value)
                        }
                    })
                    .chain(optional_prefix_iter);
                stmt.execute(params_from_iter(params)).unwrap();
                i += 1;
            }
        }
        tx.commit()?;
    }
    Ok(())
}

fn import_shapes(archive: &ZipArchive, prefix: Option<String>, db: &mut Connection) -> Result<(), Box<dyn Error>> {
    let dir = as_tree(archive.entries())?;
    let shape_file = archive.read(dir.lookup("shapes.txt")?)?;
    let mut rdr = csv::Reader::from_reader(BufReader::new(shape_file));

    rdr.deserialize::<ShapeFileRecord>().flatten()
        // group runs of shape IDs
        .group_by(|row| row.shape_id.clone()).into_iter()
        // add prefix to ID and encode polyline
        .map(move |(shape_id, rows)| (
            prefix.as_ref().map_or(shape_id.clone(), |prefix| format!("{}{}", prefix, shape_id)),
            polyline::encode_coordinates(rows.map(|row| Coord {y: row.shape_pt_lat, x: row.shape_pt_lon }), 5).unwrap()
        ))
        .chunks(1000).into_iter()
        .for_each(|shapes_to_write| { write_shapes_to_db(db, shapes_to_write) });
    Ok(())
}

#[inline]
fn write_shapes_to_db(conn: &mut Connection, shapes_to_write: impl Iterator<Item=(String, String)>) {
    let tx = conn.transaction().unwrap();
    {
        let mut stmt = tx.prepare("INSERT INTO shapes (shape_id, polyline) VALUES (?, ?)").unwrap();
        shapes_to_write.for_each(|s| {
            stmt.execute(s);
        });
    }
    tx.commit().unwrap();
}

#[derive(Deserialize)]
struct ShapeFileRecord {
    shape_id: String,
    shape_pt_lat: f64,
    shape_pt_lon: f64,
    shape_pt_sequence: u64
}

const EMPTY_SLICE: &[u8] = &[];
type Imports<'a, 'b> = [(&'a str, &'a str, Vec<&'a str>, phf::Map<&'a str, &'a str>, bool); 6];