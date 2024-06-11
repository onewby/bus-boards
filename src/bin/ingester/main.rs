use std::{fs, io, iter};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::iter::Iterator;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use geo_types::Coord;
use itertools::Itertools;
use memmap::Mmap;
use phf::phf_map;
use piz::read::{as_tree, DirectoryContents, FileTree};
use piz::ZipArchive;
use rusqlite::{Connection, params, params_from_iter};
use serde::{Deserialize, Serialize};
use suppaftp::{NativeTlsConnector, NativeTlsFtpStream};
use suppaftp::native_tls::TlsConnector;

use BusBoardsServer::config::{LastUpdates, save_updates};

type Imports<'a, 'b> = [(&'a str, &'a str, Vec<&'a str>, phf::Map<&'a str, &'a str>, bool); 6];

const SQL_INDEXES: &str = include_str!("sql/indexes.sql");
const SQL_MODEL: &str = include_str!("sql/model.sql");
const GTFS_TABLES: [&str; 6] = ["stop_times", "trips", "calendar_dates", "calendar", "routes", "agency"];

struct Source<'a> {
    name: &'a str,
    prefix: &'a str,
    url: &'a str,
    path: &'a str
}

const SOURCES: [Source; 2] = [
    Source {
        name: "BODS (approx ~550MB)",
        prefix: "",
        url: "https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/",
        path: "gtfs/itm_all_gtfs.zip",
    },
    Source {
        name: "Ember",
        prefix: "E",
        url: "https://api.ember.to/v1/gtfs/static/",
        path: "gtfs/ember.zip",
    }
];

fn main() {
    let mut connection = open_db().expect("DB init error");
    create_tables(&connection).expect("Table create error");
    clear_tables(&connection).expect("Table reconstruction error");
    for source in SOURCES {
        process_source(&source).expect("Download error");
    }
    create_indexes(&mut connection).expect("Index creation error");
    cleanup(&mut connection).expect("Cleanup error");
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

fn process_source(source: &Source) -> Result<(), Box<dyn Error>> {
    // Download source
    download_source(source)?;
    // Import zip
    import_zip(source)
}

fn download_source(source: &Source) -> Result<(), Box<dyn Error>> {
    println!("Downloading {}", source.name);
    let md = fs::metadata(&source.path);
    if md.is_ok() && SystemTime::now().duration_since(md.unwrap().modified()?)?.as_secs() < (24 * 60 * 60) {
        println!("- {} is still new - skipping.", source.path);
        return Ok(());
    }
    let resp = reqwest::blocking::Client::builder().timeout(Some(Duration::from_secs(10*60))).build()?.get(source.url).send()?;
    fs::create_dir_all("gtfs")?;
    let mut file = File::create(&source.path)?;
    let bytes = resp.bytes()?;
    file.write_all(&bytes)?;
    Ok(())
}

fn import_zip(source: &Source) -> Result<(), Box<dyn Error>> {
    let path = Path::new(source.path);
    let timer = Instant::now();
    let imports: Imports = [
        ("agency.txt", "REPLACE INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang) VALUES (?, ?, ?, ?, ?)",
         vec!["agency_id", "agency_name", "agency_url", "agency_timezone", "agency_lang"], phf_map! {}, false),
        ("routes.txt", "REPLACE INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type) VALUES (?, ?, ?, ?, ?)",
         vec!["route_id", "agency_id", "route_short_name", "route_long_name", "route_type"], phf_map! {"agency_id" => "Ember"}, false),
        ("calendar.txt", "REPLACE INTO calendar (service_id, start_date, end_date, validity) VALUES (?11||?1, ?2, ?3, (?4 + (?5 << 1) + (?6 << 2) + (?7 << 3) + (?8 << 4) + (?9 << 5) + (?10 << 6)))",
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
    let mut thread_conn = open_db()?;
    let file_name = path.file_name().unwrap().to_str().unwrap();

    for (subfile_name, stmt, indexes, defaults, add_prefix) in imports {
        println!("Importing {} for {}", subfile_name, file_name);
        import_txt_file(&archive, &dir, subfile_name, &mut thread_conn, stmt, &indexes, &defaults, if add_prefix { Some(source.prefix.as_bytes()) } else { None }).expect(subfile_name);
    }

    println!("Importing shapes.txt for {}", file_name);
    import_shapes(&archive, None, &mut thread_conn)?;

    println!("{}s to import {}", timer.elapsed().as_secs(), file_name);

    Ok(())
}

fn import_txt_file(archive: &ZipArchive, dir: &DirectoryContents, file_name: &str, db: &mut Connection, stmt_str: &str, indexes: &Vec<&str>, defaults: &phf::Map<&str, &str>, prefix: Option<&[u8]>) -> Result<(), Box<dyn Error>> {
    let file = dir.lookup(file_name)?;
    let stream_reader = BufReader::new(archive.read(file)?);
    let mut rdr = csv::Reader::from_reader(stream_reader);
    let headers = rdr.headers()?.clone();
    let headers: HashMap<_, _> = headers.iter().enumerate().map(|(i, hdr)| (hdr, i)).collect();
    let mut record = csv::ByteRecord::new();
    while !rdr.is_done() {
        let mut tx = db.transaction()?;
        {
            let mut stmt = tx.prepare(stmt_str)?;
            let mut i: u32 = 0;
            while i < 10000 && rdr.read_byte_record(&mut record)? {
                let optional_prefix_iter: Box<dyn Iterator<Item=Option<&[u8]>>> = match prefix {
                    None => Box::new(iter::empty()),
                    Some(prefix) => Box::new(iter::once(Some(prefix)))
                };
                let params = indexes.iter()
                    .map(|hdr_name| {
                        let located_value = headers.get(hdr_name).and_then(|i| record.get(*i)).unwrap_or(EMPTY_SLICE);
                        if located_value.is_empty() {
                            defaults.get(hdr_name).map(|def| def.as_bytes())
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
            polyline::encode_coordinates(rows.map(|row| Coord {x: row.shape_pt_lat, y: row.shape_pt_lon }), 5).unwrap()
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

fn create_indexes(conn: &mut Connection) -> rusqlite::Result<usize> {
    println!("Creating indexes");
    conn.execute(SQL_INDEXES, [])
}

#[derive(Debug, Deserialize, Serialize)]
struct ServiceReportRecord {
    #[serde(rename = "NationalOperatorCode")]
    national_operator_code: String,
    #[serde(rename = "LineName")]
    line_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgencyInfo {
    agency_id: String,
    agency_name: String,
    routes: String,
    code: Option<String>,
    website: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublicNameRecord {
    #[serde(rename = "PubNmId")]
    pub_nm_id: String,
    #[serde(rename = "Website")]
    website: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NOCTableRecord {
    #[serde(rename = "NOCCODE")]
    noccode: String,
    #[serde(rename = "PubNmId")]
    pub_nm_id: String,
    #[serde(rename = "OperatorPublicName")]
    operator_public_name: String,
    #[serde(rename = "VOSA_PSVLicenseName")]
    vosa_psv_license_name: String,
}

fn get_service_report() -> Result<io::Cursor<Vec<u8>>, Box<dyn Error>> {
    let ftp_host = "ftp.tnds.basemap.co.uk";
    let ftp_user = std::env::var("TNDS_USERNAME")?;
    let ftp_password = std::env::var("TNDS_PASSWORD")?;

    // FTP Access
    let ftp_stream = NativeTlsFtpStream::connect((ftp_host, 21))?;
    let mut ftp_stream = ftp_stream.into_secure(NativeTlsConnector::from(TlsConnector::new()?), ftp_host)?;
    ftp_stream.login(&ftp_user, &ftp_password)?;
    println!("Connected to FTP");

    // Download the file from FTP
    let service_report_file = ftp_stream.retr_as_buffer("servicereport.csv")?;
    ftp_stream.quit()?;
    Ok(service_report_file)
}

fn download_noc(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let service_report_file = get_service_report()?;
    println!("Downloaded TNDS service report");

    // Download Traveline zip file
    let traveline_url = "https://www.travelinedata.org.uk/wp-content/themes/desktop/nocadvanced_download.php?reportFormat=csvFlatFile&allTable%5B%5D=table_noc_table&allTable%5B%5D=table_public_name&submit=Submit";
    let traveline_zip = reqwest::blocking::get(traveline_url)?.bytes()?;
    let mut archive = ZipArchive::new(traveline_zip.as_ref())?;
    let dir = as_tree(archive.entries())?;
    let pnr_file = archive.read(dir.lookup("PublicName.csv")?)?;
    let noc_file = archive.read(dir.lookup("NOCTable.csv")?)?;

    let mut public_name_records = csv::Reader::from_reader(BufReader::new(pnr_file)).deserialize::<PublicNameRecord>().flatten().collect_vec();
    let mut noc_records = csv::Reader::from_reader(BufReader::new(noc_file));
    let noc_table: HashMap<_, _> = noc_records.deserialize::<NOCTableRecord>().flatten()
        .group_by(|r| r.noccode.clone()).into_iter().map(|(k, mut v)| (k, v.next().unwrap())).collect();

    // Read service report records
    let service_report_records: Vec<ServiceReportRecord> = csv::Reader::from_reader(BufReader::new(service_report_file)).deserialize().flatten().collect_vec();
    let nocs = service_report_records.iter().into_group_map_by(|r| r.national_operator_code.clone());
    let route_nocs: HashMap<_, _> = nocs.iter().map(|(noc, records)| (records.iter().map(|&r| r.line_name.clone()).sorted().collect::<Vec<_>>().join(","), noc)).collect();

    // Get agency info, with Traveline code where possible
    let mut assigned_codes = conn.prepare(
        "SELECT agency.agency_id, agency.agency_name, (SELECT group_concat(route_short_name) FROM routes WHERE routes.agency_id=agency.agency_id ORDER BY route_short_name) as routes FROM agency"
    )?.query_map([], |row| {
        Ok(AgencyInfo {
            agency_id: row.get("agency_id")?,
            agency_name: row.get("agency_name")?,
            routes: row.get("routes")?,
            code: route_nocs.get(row.get::<_, String>("routes")?.as_str()).map(|s| s.to_string()),
            website: None,
        })
    })?.flatten().collect_vec();

    let assigned_code_strs: HashSet<String> = assigned_codes.iter().filter_map(|a| a.code.clone()).collect();
    let remaining_agencies: Vec<_> = assigned_codes.iter_mut().filter(|a| a.code.is_none()).collect();
    let remaining_traveline: Vec<(String, String)> = nocs.iter()
        .filter(|(k, _)| !assigned_code_strs.contains(*k))
        .map(|(k, v)| {
            (
                k.clone(),
                v.iter().map(|r| r.line_name.clone()).sorted().collect::<Vec<_>>().join(",")
            )
        })
        .collect();

    for agency in remaining_agencies {
        let cands: Vec<_> = remaining_traveline.iter()
            .filter(|(t_k, _)| agency.agency_name == noc_table.get(t_k).map_or("", |r| &r.operator_public_name))
            .collect();

        if cands.is_empty() {
            let last_ditch_cands: Vec<_> = noc_table.values()
                .filter(|t| agency.agency_name == t.operator_public_name)
                .collect();
            if last_ditch_cands.len() == 1 {
                agency.code = Some(last_ditch_cands[0].noccode.clone());
            } else {
                println!("Could not map {} to Traveline data", agency.agency_name);
            }
            continue;
        }

        let closest = cands.iter().min_by(|(_, t_v1), (_, t_v2)| {
            let sim1 = sorensen::distance(agency.routes.as_bytes(), t_v1.as_bytes());
            let sim2 = sorensen::distance(agency.routes.as_bytes(), t_v2.as_bytes());
            sim2.partial_cmp(&sim1).unwrap()
        });
        if let Some((t_k, _)) = closest {
            agency.code = Some(t_k.clone());
        }
    }

    assigned_codes
        .iter_mut()
        .filter(|a| a.code.is_some())
        .for_each(|assignment| {
            if let Some(noc) = noc_table.get(&assignment.code.clone().unwrap()) {
                if let Some(pnr) = public_name_records.iter().find(|t| t.pub_nm_id == noc.pub_nm_id) {
                    let first_index = pnr.website.find('#');
                    let last_index = pnr.website.rfind('#');
                    assignment.website = if let (Some(first), Some(last)) = (first_index, last_index) {
                        Some(pnr.website[first + 1..last].to_string())
                    } else {
                        Some(pnr.website.clone())
                    }
                }
            }
        });

    // Insert into SQLite database
    conn.execute("DELETE FROM traveline", params![])?;

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO traveline (agency_name, routes, code, website) VALUES (?1, ?2, ?3, ?4)")?;
        for record in &assigned_codes {
            stmt.execute(params![
                &record.agency_name,
                &record.routes,
                &record.code,
                &record.website,
            ])?;
        }
    }
    tx.commit()?;

    Ok(())
}

#[derive(Deserialize)]
struct ShapeFileRecord {
    shape_id: String,
    shape_pt_lat: f64,
    shape_pt_lon: f64,
    shape_pt_sequence: u64
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

fn clean_arrivals(db: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Cleaning up arrivals");
    let json_str = fs::read_to_string("localities.json").expect("Cannot find localities.json");
    let localities: Localities = serde_json::from_str(&json_str).expect("JSON parse fail");
    let mut arrival_bays: Vec<&Stance> = Vec::new();
    for stop in localities.values() {
        for stances in stop.values() {
            arrival_bays.extend(stances.iter().filter(|st| st.arrival));
        }
    }
    let arrival_list: String = arrival_bays.iter().map(|bay| format!("'{}'", bay.atco_code)).collect::<Vec<String>>().join(",");
    let select_all = format!("SELECT arrival.ROWID as arr_id, arrival.arrival_time as arrival_time, departure.ROWID as dep_id FROM stop_times AS departure
      INNER JOIN stances st1 on departure.stop_id = st1.code
      INNER JOIN stop_times arrival on arrival.stop_sequence=departure.stop_sequence-1 AND arrival.stop_id IN ({arrival_list})
      INNER JOIN stances st2 on st2.code = arrival.stop_id
      INNER JOIN trips t on t.trip_id = departure.trip_id
    WHERE st1.stop == st2.stop AND departure.trip_id=arrival.trip_id;");
    let mut stmt = db.prepare(&select_all)?;
    let arrivals: Vec<ArrivalsSelectResult> = stmt.query_map([], |row| {
        Ok(ArrivalsSelectResult {
            arr_id: row.get(0)?,
            arr_time: row.get(1)?,
            dep_id: row.get(2)?,
        })
    })?.filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
    stmt.finalize()?;
    for arrival in arrivals {
        let tx = db.transaction()?;
        tx.prepare_cached("UPDATE stop_times SET arrival_time=? WHERE rowid=?")?.execute(params!(arrival.arr_time, arrival.dep_id))?;
        tx.prepare_cached("DELETE FROM stop_times WHERE rowid=?")?.execute([arrival.arr_id])?;
        tx.commit()?;
    }
    Ok(())
}

struct ArrivalsSelectResult {
    arr_id: i32,
    arr_time: String,
    dep_id: i32
}

fn clean_stops(conn: &Connection) -> Result<(), rusqlite::Error> {
    println!("Cleaning up stops");
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    conn.execute("DELETE FROM stops WHERE stops.id NOT IN (SELECT DISTINCT stances.stop FROM stop_times INNER JOIN stances ON stances.code=stop_id);", [])?;
    println!("Rebuilding stops_search");
    // Rebuild stops_search table
    conn.execute("DROP TABLE IF EXISTS stops_search;", [])?;
    conn.execute("CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED);", [])?;
    conn.execute("INSERT INTO stops_search(name, parent, qualifier, id) SELECT stops.name, stops.locality_name, qualifier, stops.id FROM stops INNER JOIN localities l on l.code = stops.locality;", [])?;
    Ok(())
}

fn reset_polar() -> Result<(), Box<dyn Error>> {
    save_updates(LastUpdates::default())
}

fn patch_display_names(conn: &mut Connection) -> rusqlite::Result<usize> {
    println!("Patching route display names");
    conn.execute("UPDATE routes SET route_short_name=route_id WHERE agency_id='Ember'", [])?;
    conn.execute(
        r#"UPDATE trips SET trip_headsign=(SELECT CASE
                    WHEN original = 'Tokyngton' THEN 'Wembley Stadium'
                    WHEN original = 'Centenary Square' THEN 'Birmingham'
                    WHEN original = 'Penglais' THEN 'Aberystwyth University'
                    WHEN original = 'Causewayhead' THEN 'University of Stirling'
                    WHEN origin_loc = dest_loc THEN original
                    WHEN dest_loc = 'London' THEN dest_loc || ' ' || original
                    ELSE dest_loc END)
                FROM (SELECT trips.trip_id,
                         (SELECT IFNULL(substr(s.locality_name, 0, NULLIF(instr(s.locality_name, '›') - 1, -1)), s.locality_name)
                            FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=dest.stop_id) as dest_loc,
                         (SELECT IFNULL(substr(s.locality_name, 0, NULLIF(instr(s.locality_name, '›') - 1, -1)), s.locality_name)
                            FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=origin.stop_id) as origin_loc,
                          trip_headsign AS original FROM trips
                  INNER JOIN main.routes r on r.route_id = trips.route_id
                  INNER JOIN main.stop_times origin on (trips.trip_id = origin.trip_id AND trips.min_stop_seq=origin.stop_sequence)
                  INNER JOIN main.stop_times dest on (trips.trip_id = dest.trip_id AND trips.max_stop_seq=dest.stop_sequence)
                WHERE agency_id IN ('OP5050', 'OP564', 'OP5051', 'OP545', 'OP563') AND NOT instr(trip_headsign, 'Airport')) AS trip_subquery
                WHERE trips.trip_id=trip_subquery.trip_id"#, [])
}

fn remove_traveline_ember(conn: &mut Connection) -> rusqlite::Result<usize> {
    println!("Removing Ember TNDS data");
    conn.execute("DELETE FROM stop_times WHERE trip_id=(SELECT trip_id FROM trips INNER JOIN main.routes r on r.route_id = trips.route_id WHERE r.agency_id='OP965')", [])?;
    conn.execute("DELETE FROM trips WHERE trip_id=(SELECT trip_id FROM trips INNER JOIN main.routes r on r.route_id = trips.route_id WHERE r.agency_id='OP965')", [])?;
    conn.execute("DELETE FROM routes WHERE agency_id='OP965'", [])
}

fn cleanup(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    conn.execute("UPDATE trips SET max_stop_seq=(SELECT max(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)", ())?;
    clean_arrivals(conn).expect("Clean arrivals");
    clean_stops(conn).expect("Clean stops");
    reset_polar().expect("Reset Polar");
    patch_display_names(conn).expect("Display name patching");
    remove_traveline_ember(conn).expect("Patch Ember");
    Ok(())
}

const EMPTY_SLICE: &[u8] = &[];