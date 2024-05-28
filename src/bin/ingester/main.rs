mod pipeline;

use std::{fs, io, thread};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File};
use std::io::{BufReader, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use piz::read::{as_tree, DirectoryContents, FileTree};
use piz::ZipArchive;
use rusqlite::{Connection, params, params_from_iter};
use memmap::Mmap;
use serde::{Deserialize, Serialize};

type Imports<'a> = [(&'a str, &'a str, Vec<usize>); 6];

fn main() {
    let mut connection = open_db().expect("DB init error");
    create_tables(&connection).expect("Table create error");
    let initial_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time error").as_secs() - 1;
    for source in SOURCES {
        process_source(&source).expect("Download error");
    }
    import_zips().expect("Import error");
    cleanup(&mut connection, &initial_time).expect("Cleanup error");
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
    let statements = fs::read_to_string("src/model.sql")?;
    conn.execute_batch(&*statements)?;
    Ok(())
}

fn process_source(source: &Source) -> Result<(), Box<dyn Error>> {
    // Download source
    download_source(source)?;
    // Import zip
    import_zip(Path::new(&source.path))
}

fn import_zips() -> Result<(), io::Error> {
    let zips = get_zips()?;
    let handles: Vec<JoinHandle<()>> = zips.into_iter()
        .map(|zip| thread::spawn(move || {
            import_zip(zip.as_path()).expect("Import error")
        })).collect();
    for handle in handles {
        let _ = handle.join();
    }
    Ok(())
}

fn import_zip(path: &Path) -> Result<(), Box<dyn Error>> {
    let imports: Imports = [
        ("agency.txt", "REPLACE INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang, modified) VALUES (?, ?, ?, ?, ?, unixepoch())", (0..5).collect()),
        ("routes.txt", "REPLACE INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type, modified) VALUES (?, ?, ?, ?, ?, unixepoch())", (0..5).collect()),
        ("calendar.txt", "REPLACE INTO calendar (service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date, modified) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())", (0..10).collect()),
        ("calendar_dates.txt", "REPLACE INTO calendar_dates (service_id, date, exception_type, modified) VALUES (?, ?, ?, unixepoch())", (0..3).collect()),
        ("trips.txt", "REPLACE INTO trips (route_id, service_id, trip_id, trip_headsign, modified) VALUES (?, ?, ?, ?, unixepoch())", (0..4).collect()),
        ("stop_times.txt", "REPLACE INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, stop_headsign, pickup_type, drop_off_type, timepoint, modified) VALUES (?, ?, ?, ?, ?, NULLIF(?, ''), ?, ?, ?, unixepoch())", [0, 1, 2, 3, 4, 5, 6, 7, 9].to_vec())
    ];

    let zip_file = File::open(path)?;
    let mapping = unsafe { Mmap::map(&zip_file)? };
    let archive = ZipArchive::new(&mapping)?;
    let dir = as_tree(archive.entries())?;
    let thread_conn = open_db()?;
    let file_name = path.file_name().unwrap().to_str().unwrap();

    for import in imports {
        println!("Importing {} for {}", import.0, file_name);
        import_txt_file(&archive, &dir, import.0, &thread_conn, import.1, &import.2).expect(import.0);
    }

    Ok(())
}

fn import_txt_file(archive: &ZipArchive, dir: &DirectoryContents, file_name: &str, db: &Connection, stmt: &str, indexes: &Vec<usize>) -> Result<(), Box<dyn Error>> {
    let mut stmt = db.prepare(stmt)?;
    let file = dir.lookup(file_name)?;
    let stream_reader = BufReader::new(archive.read(file)?);
    let mut rdr = csv::Reader::from_reader(stream_reader);
    let mut record = csv::ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
        let params = indexes.iter().map(|i| record.get(*i).unwrap()).map(|x| if x.is_empty() {None} else {Some(x)});
        stmt.execute(params_from_iter(params)).unwrap();
    }
    Ok(())
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
    street: String,
    #[serde(rename = "Indicator")]
    indicator: String,
    #[serde(rename = "Arrival")]
    arrival: bool
}

fn clean_arrivals(db: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Cleaning up arrivals");
    let json_str = fs::read_to_string("../localities.json")?;
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

fn cleanup(conn: &mut Connection, initial_time: &u64) -> Result<(), Box<dyn Error>> {
    conn.execute("UPDATE trips SET max_stop_seq=(SELECT max(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)", ())?;
    clean_arrivals(conn).expect("Clean arrivals");
    clean_stops(conn).expect("Clean stops");
    println!("Removing old data");
    let table_names = ["agency", "routes", "calendar", "calendar_dates", "trips", "stop_times"];
    for table in table_names {
        conn.execute(&*format!("DELETE FROM {} WHERE modified <= ?", table), [initial_time])?;
    }
    Ok(())
}

fn get_zips() -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir("gtfs")?
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_file()
            && x.file_name().to_ascii_lowercase().to_str().unwrap().ends_with(".zip"))
        .map(|x| x.path())
        .collect())
}

struct Source {
    name: String,
    prefix: String,
    url: String,
    path: String
}

const SOURCES: [Source; 2] = [
    Source {
        name: "BODS (approx ~550MB)".to_string(),
        prefix: "".to_string(),
        url: "https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/".to_string(),
        path: "gtfs/itm_all_gtfs.zip".to_string(),
    },
    Source {
        name: "Ember".to_string(),
        prefix: "E".to_string(),
        url: "https://api.ember.to/v1/gtfs/static/".to_string(),
        path: "gtfs/ember.zip".to_string(),
    }
];

fn download_source(source: &Source) -> Result<(), Box<dyn Error>> {
    println!("Downloading {}", source.name);
    let md = fs::metadata(&source.path);
    if md.is_ok() && SystemTime::now().duration_since(md.unwrap().modified()?)?.as_secs() < (24 * 60 * 60) {
        println!("- {} is still new - skipping.", source.path);
        return Ok(());
    }
    let resp = reqwest::blocking::get(&source.url)?;
    fs::create_dir_all("gtfs")?;
    let mut file = File::create(&source.path)?;
    let bytes = resp.bytes()?;
    file.write_all(&bytes)?;
    Ok(())
}