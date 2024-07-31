use serde_nested_with::serde_nested;
use std::collections::HashMap;
use std::ops::Add;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use chrono::{Datelike, DateTime, Duration, DurationRound, TimeDelta, Utc};
use geo_types::{Coord, coord, Point};
use itertools::Itertools;
use memoize::memoize;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rand::Rng;
use rusqlite::{named_params, Params, params};
use rusqlite::types::Value;
use serde::{de, Deserialize, Deserializer, Serializer};
use serde_with::serde_as;
use serde_with::DurationSeconds;

use BusBoardsServer::config::BBConfig;

use crate::bus_prediction::TripCandidate;
use crate::passenger::PassengerDirectionInfo;
use crate::util::{adjust_timestamp, gtfs_date, relative_to, zero_day, zero_time};

pub type DBPool = Pool<SqliteConnectionManager>;
pub type PooledConn = PooledConnection<SqliteConnectionManager>;

/// Create database connection pool
pub fn open_db() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file("stops.sqlite")
        .with_init(|s| rusqlite::vtab::array::load_module(s));
    Pool::new(manager).unwrap_or_else(|e| open_db())
}

/// Get connection from database pool
pub fn get_pool(db: &Arc<DBPool>) -> PooledConn {
    let mut conn: Result<PooledConnection<SqliteConnectionManager>, r2d2::Error>;
    while {
        conn = db.get();
        if conn.is_err() {
            sleep(std::time::Duration::from_millis(rand::thread_rng().gen_range(500..1500)))
        }
        conn.is_err()
    } {}
    conn.unwrap()
}

/// Get single string result from database query
pub fn get_string<P: Params>(db: &Arc<DBPool>, query: &str, params: P) -> Result<String, String> {
    get_pool(db).prepare_cached(query).unwrap().query_row(params, |aid| aid.get(0)).map_err(|e| e.to_string())
}

/// Traveline operator code -> GTFS agency ID
#[memoize(Ignore: db)]
pub fn get_agency(db: &Arc<DBPool>, code: String) -> Result<String, String> {
    get_string(db, "SELECT agency_id FROM traveline WHERE code=?", params![code])
}

/// Traveline operator code + route name -> GTFS route ID
#[memoize(Ignore: db)]
pub fn get_route(db: &Arc<DBPool>, code: String, route: String) -> Result<String, String> {
    get_string(db, "SELECT route_id FROM routes INNER JOIN main.traveline t on routes.agency_id = t.agency_id WHERE code=? AND route_short_name=?", (code, route))
}

/// GTFS agency ID and route name -> GTFS route ID
#[memoize(Ignore: db)]
pub fn get_route_id(db: &Arc<DBPool>, agency_id: String, route: String) -> Result<String, String> {
    get_string(db, "SELECT route_id FROM routes WHERE agency_id=? AND upper(route_short_name)=upper(?)", (agency_id, route))
}

/// Get trip candidates for realtime vehicle matching - query made specific to data provider
fn trip_query(query: &str, db: &Arc<DBPool>, date: &DateTime<Utc>, start_before: i64, end_after: i64, specifier: &str) -> Vec<TripCandidate>  {
    let date_secs = zero_time(date).timestamp();

    get_pool(db).prepare_cached(query).unwrap().query_map(named_params! {
        ":date": u64::from_str(date.format("%Y%m%d").to_string().as_str()).unwrap(),
        ":day": date.weekday().num_days_from_monday(),
        ":startTime": start_before,
        ":endTime": end_after,
        ":specifier": specifier
    }, |row| {
        let route_str: String = row.get("route")?;
        let times_str: String = row.get("times")?;
        let seqs_str: String = row.get("seqs")?;

        Ok(TripCandidate {
            trip_id: row.get("trip_id")?,
            direction: row.get("direction").ok(),
            route: route_str.split(',').map(|s| s.to_string()).collect(),
            times: times_str.split(',').map(|s| DateTime::from_timestamp(i64::from_str(s).unwrap() + date_secs, 0).unwrap()).collect(),
            seqs: seqs_str.split(',').map(|s| u32::from_str(s).unwrap()).collect(),
            date: row.get("date")?,
        })
    }).unwrap().filter_map(|i| i.ok()).collect()
}

/// Get trip candidates for Passenger realtime vehicle matching
pub fn passenger_trip_query(db: &Arc<DBPool>, date: &DateTime<Utc>, start_before: i64, end_after: i64, route_id: &str) -> Vec<TripCandidate> {
    trip_query(r#"SELECT trips.trip_id, p.direction, :date as date,
                (SELECT group_concat(stop_id) FROM (SELECT stop_id FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as route,
                (SELECT group_concat(departure_time) FROM (SELECT departure_time FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as times,
                (SELECT group_concat(stop_sequence) FROM (SELECT stop_sequence FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as seqs
                 FROM trips
                          INNER JOIN main.routes r on r.route_id = trips.route_id
                          LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                          LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
                          LEFT OUTER JOIN main.polar p on trips.trip_id = p.gtfs
                          INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                          INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
                 WHERE r.route_id=:specifier
                   AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                   AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                   AND +start.departure_time <= :startTime AND +finish.departure_time >= :endTime
    "#, db, date, start_before, end_after, route_id)
}

/// Get trip candidates for Lothian realtime vehicle matching
pub fn lothian_trip_query(db: &Arc<DBPool>, date: &DateTime<Utc>, start_before: i64, end_after: i64, pattern: &str) -> Vec<TripCandidate> {
    trip_query(r#"SELECT trips.trip_id, :date as date,
                (SELECT group_concat(stop_id) FROM (SELECT stop_id FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as route,
                (SELECT group_concat(departure_time) FROM (SELECT departure_time FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as times,
                (SELECT group_concat(stop_sequence) FROM (SELECT stop_sequence FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as seqs
                 FROM polar
                   INNER JOIN main.trips trips on polar.gtfs = trips.trip_id
                   LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                   LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
                   INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                   INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
                 WHERE direction IS NULL AND polar=:specifier
                   AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                   AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                   AND +start.departure_time <= :startTime AND +finish.departure_time >= :endTime
    "#, db, date, start_before, end_after, pattern)
}

/// GTFS route -> coordinates for each stance on route
#[memoize(Ignore: db)]
pub fn get_line_segments(db: &Arc<DBPool>, route_id: String) -> HashMap<String, Point> {
    HashMap::from_iter(get_pool(db).prepare_cached(
        r#"SELECT DISTINCT code as stop_id, lat as y, long as x FROM stances
                WHERE code IN (
                   SELECT stop_id FROM stop_times
            INNER JOIN main.trips t on t.trip_id = stop_times.trip_id WHERE t.route_id=?)"#).unwrap()
        .query_map(params![route_id], |result| Ok((result.get("stop_id")?, Point::new(result.get("x")?, result.get("y")?)))).unwrap().filter_map(|v| v.ok()))
}

/// Route pattern -> GTFS route
#[derive(Clone)]
pub struct LothianDBPattern {
    pub(crate) route: String,
    pub(crate) pattern: String
}

/// Get list of route pattern -> GTFS route mappings
pub fn get_lothian_patterns_tuples(db: &Arc<DBPool>) -> Vec<LothianDBPattern> {
    get_pool(db).prepare_cached(r#"SELECT * FROM lothian"#).unwrap()
        .query_map(params![], |row| Ok(LothianDBPattern {route: row.get("route")?, pattern: row.get("pattern")? })).unwrap()
        .filter_map(|x: Result<LothianDBPattern, _>| x.ok()).collect()
}

/// Lothian route name -> GTFS route ID
#[memoize(Ignore: db)]
pub fn get_lothian_route(db: &Arc<DBPool>, route: String) -> Option<String> {
    get_pool(db).prepare_cached("SELECT route_id FROM routes WHERE route_short_name=? AND agency_id IN (\"OP596\", \"OP597\", \"OP598\", \"OP549\")").unwrap()
        .query_row(params![route], |row| row.get("route_id")).ok()
}

/// GTFS trip <-> Stagecoach feed journey mapping
#[derive(Clone)]
pub struct StagecoachRoute {
    pub trip_id: String,
    pub route_id: String,
    pub stop_seq: u64,
    pub stop_id: String
}

/// Find GTFS trip match from Stagecoach realtime journey
pub fn get_stagecoach_trip(db: &Arc<DBPool>, agency_id: &str, route_name: &str, next_stop: &str, departure: &DateTime<Utc>) -> Option<StagecoachRoute> {
    get_pool(db).prepare_cached(r#"
        SELECT t.trip_id, r.route_id, stop_times.stop_sequence as stop_seq, stop_times.stop_id FROM stop_times
            INNER JOIN trips t on t.trip_id = stop_times.trip_id
            INNER JOIN main.routes r on t.route_id = r.route_id
            INNER JOIN stop_times origin on (origin.trip_id=t.trip_id AND origin.stop_sequence=t.min_stop_seq)
            LEFT OUTER JOIN main.calendar c on c.service_id = t.service_id
            LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
        WHERE agency_id = :agency_id AND route_short_name = :route_name AND stop_times.stop_id=:next_stop
          AND origin.departure_time = :dep_time
          AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
          AND NOT (exception_type IS NOT NULL AND exception_type = 2)
    "#).unwrap().query_row(named_params! {
            ":date": u64::from_str(departure.format("%Y%m%d").to_string().as_str()).unwrap(),
            ":day": departure.weekday().num_days_from_monday(),
            ":dep_time": zero_day(&departure).duration_trunc(TimeDelta::minutes(1)).unwrap().timestamp(),
            ":agency_id": agency_id,
            ":route_name": route_name,
            ":next_stop": next_stop
        }, |row| {
        Ok(StagecoachRoute {
            trip_id: row.get("trip_id")?,
            route_id: row.get("route_id")?,
            stop_seq: row.get("stop_seq")?,
            stop_id: row.get("stop_id")?,
        })
    }).ok()
}

/// GTFS route
#[derive(Clone)]
pub struct CoachRoute {
    pub agency_id: String,
    pub route_id: String,
    pub route_short_name: String
}

/// Get list of coach routes
pub fn get_coach_routes(db: &Arc<DBPool>, config: &Arc<BBConfig>) -> Vec<CoachRoute> {
    let values = Rc::new(config.coaches.operators.iter().cloned().map(Value::from).collect::<Vec<Value>>());
    get_pool(db).prepare("SELECT agency_id, route_id, route_short_name FROM routes WHERE agency_id IN (SELECT value from rarray(?1))").unwrap()
        .query_map([values], |row| {
            let route_short_name: String = row.get("route_short_name")?;
            Ok(CoachRoute {
                agency_id: row.get("agency_id")?,
                route_id: row.get("route_id")?,
                route_short_name: config.coaches.route_overrides.get(route_short_name.as_str()).unwrap_or(&route_short_name).to_string(),
            })
    }).unwrap().filter_map(|c| c.ok()).collect_vec()
}

/// GTFS trip info
pub struct CoachTrip {
    pub trip_id: String,
    pub route: Vec<String>,
    pub times: Vec<usize>,
    pub seqs: Vec<usize>,
    pub std_loc: String,
    pub sta_loc: String
}

/// Coach realtime journey -> GTFS trip
pub fn get_coach_trip(db: &Arc<DBPool>, route: &str, origin: &str, dest: &str, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Option<CoachTrip> {
    get_pool(db).prepare_cached(
        r#"SELECT trips.trip_id,
            (SELECT group_concat(stop_id) FROM (SELECT stop_id FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as route,
            (SELECT group_concat(departure_time) FROM (SELECT departure_time FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as times,
            (SELECT group_concat(stop_sequence) FROM (SELECT stop_sequence FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as seqs,
            (SELECT locality_name FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=std.stop_id) as stdLoc,
            (SELECT locality_name FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=sta.stop_id) as staLoc
            FROM trips
                INNER JOIN main.stop_times std on (trips.trip_id = std.trip_id AND std.stop_sequence=min_stop_seq)
                INNER JOIN main.stop_times sta on (trips.trip_id = sta.trip_id AND sta.stop_sequence=max_stop_seq)
                LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
            WHERE route_id=:route AND +std.departure_time=:startTime AND +sta.departure_time=:endTime
                AND stdLoc LIKE :depWildcard AND staLoc LIKE :arrWildcard
                AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                    AND NOT (exception_type IS NOT NULL AND exception_type = 2)"#).unwrap()
        .query_row(named_params! {
            ":route": route,
            ":startTime": adjust_timestamp(&zero_day(start)).timestamp(),
            ":endTime": adjust_timestamp(&relative_to(start, end)).timestamp(),
            ":depWildcard": format!("{}%", origin.split_once('(').map(|s| s.0).unwrap_or(origin)),
            ":arrWildcard": format!("{}%", dest.split_once('(').map(|s| s.0).unwrap_or(dest)),
            ":date": usize::from_str(gtfs_date(start).as_str()).unwrap(),
            ":day": start.weekday().num_days_from_monday()
        }, |row| {
            Ok(CoachTrip {
                trip_id: row.get("trip_id")?,
                route: row.get::<_, String>("route")?.split(',').map(str::to_string).collect_vec(),
                times: row.get::<_, String>("times")?.split(',').filter_map(|s| usize::from_str(s).ok()).collect_vec(),
                seqs: row.get::<_, String>("seqs")?.split(',').filter_map(|s| usize::from_str(s).ok()).collect_vec(),
                std_loc: row.get("stdLoc")?,
                sta_loc: row.get("staLoc")?,
            })
        }).ok()
}

type TripID = String;
pub type RouteID = String;
pub type RouteName = String;

/// First realtime journey -> GTFS trip
pub fn get_first_trip(db: &Arc<DBPool>, route: &str, agency_id: &str, start_stop: &str, start_time: &DateTime<Utc>) -> Option<TripID> {
    get_pool(db).prepare_cached(
        r#"SELECT trips.trip_id FROM trips
                INNER JOIN main.routes r on r.route_id = trips.route_id
                INNER JOIN main.stop_times st on (trips.trip_id = st.trip_id AND stop_sequence=min_stop_seq)
                LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
            WHERE route_short_name=:route AND agency_id=:op AND st.stop_id=:startStop AND st.departure_time=:startTime
                AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                    AND NOT (exception_type IS NOT NULL AND exception_type = 2)"#
    ).unwrap().query_row(
        named_params![
            ":route": route,
            ":op": agency_id,
            ":startStop": start_stop,
            ":startTime": zero_day(start_time).timestamp(),
            ":date": u64::from_str(gtfs_date(start_time).as_str()).unwrap(),
            ":day": start_time.weekday().num_days_from_monday()
        ],
        |row| {
            row.get("trip_id")
        }
    ).ok()
}

/// Delete Lothian journey <-> GTFS trip mappings
pub fn reset_lothian(db: &Arc<DBPool>) {
    get_pool(db).execute("DELETE FROM polar WHERE direction IS NULL", params![]).unwrap();
}

/// GTFS agency ID -> GTFS routes
pub fn get_operator_routes(db: &Arc<DBPool>, agency_id: &str) -> Vec<(RouteID, RouteName)> {
    get_pool(db).prepare_cached("SELECT route_id,route_short_name FROM routes WHERE agency_id=?").unwrap()
        .query_map(params![agency_id], |row| Ok((row.get("route_id")?, row.get("route_short_name")?)))
        .unwrap().filter_map(|c| c.ok()).collect_vec()
}

/// GTFS trip info
pub struct LothianGTFSTrip {
    pub min_stop_time: i64,
    pub max_stop_time: i64,
    pub origin_stop: String,
    pub dest_stop: String,
    pub trip_id: String
}

/// GTFS route -> GTFS trips
pub fn get_lothian_timetabled_trips(db: &Arc<DBPool>, date: &DateTime<Utc>, route_id: &str) -> Vec<LothianGTFSTrip> {
    get_pool(db).prepare_cached(
        r#"SELECT start.departure_time as minss, start.stop_id as startStop, finish.departure_time as maxss, finish.stop_id as finishStop, trips.trip_id
            FROM trips
                LEFT OUTER JOIN main.calendar c on trips.service_id = c.service_id
                LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=:date)
                INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
            WHERE route_id=:route
              AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
              AND NOT (exception_type IS NOT NULL AND exception_type = 2)"#).unwrap()
        .query_map(named_params![
            ":route": route_id,
            ":date": gtfs_date(date),
            ":day": date.weekday().num_days_from_monday()
        ], |row| {
            Ok(LothianGTFSTrip {
                min_stop_time: row.get("minss")?,
                max_stop_time: row.get("maxss")?,
                origin_stop: row.get("startStop")?,
                dest_stop: row.get("finishStop")?,
                trip_id: row.get("trip_id")?,
            })
        }).unwrap().filter_map(|t| t.ok()).collect_vec()
}

/// Save Lothian journey -> GTFS trip mappings and Lothian pattern -> GTFS route mappings
pub fn save_lothian_pattern_allocations(db: &Arc<DBPool>, pattern: &str, trip_ids: &[String], gtfs_route_id: &str) -> Result<(), rusqlite::Error> {
    let mut db = get_pool(db);
    let tx = db.transaction()?;
    {
        let mut stmt= tx.prepare_cached("INSERT OR IGNORE INTO polar (gtfs, polar) VALUES (?,?)")?;
        for trip_id in trip_ids {
            stmt.execute(params![trip_id, pattern])?;
        }
        let mut stmt = tx.prepare_cached("INSERT INTO lothian (pattern, route) VALUES (?, ?)")?;
        stmt.execute(params![pattern, gtfs_route_id])?;
    }
    tx.commit()
}

/// Delete Passenger journey <-> GTFS trip mappings
pub fn reset_passenger(db: &Arc<DBPool>) {
    get_pool(db).execute("DELETE FROM polar WHERE direction IS NOT NULL", params![]).unwrap();
}

/// GTFS trip and its origin/destination time
pub struct PassengerRouteTrip {
    pub min_stop_time: i64,
    pub max_stop_time: i64,
    pub trip_id: String
}

/// Get GTFS route info
pub fn get_passenger_route_trips(db: &Arc<DBPool>, date: &DateTime<Utc>, route_id: &str) -> Vec<PassengerRouteTrip> {
    get_pool(db).prepare_cached(
        r#"SELECT start.departure_time as minss, finish.departure_time as maxss, trips.trip_id
            FROM trips
                LEFT OUTER JOIN main.calendar c on trips.service_id = c.service_id
                LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=:date)
                INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
            WHERE route_id=:route
              AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
              AND NOT (exception_type IS NOT NULL AND exception_type = 2)"#).unwrap()
        .query_map(named_params! {
            ":route": route_id,
            ":date": gtfs_date(date),
            ":day": date.weekday().num_days_from_monday()
        }, |row| {
            Ok(PassengerRouteTrip {
                min_stop_time: row.get("minss")?,
                max_stop_time: row.get("maxss")?,
                trip_id: row.get("trip_id")?
            })
        }).unwrap().filter_map(|t| t.ok()).collect_vec()
}

/// Save Passenger journey <-> GTFS trip mappings
pub fn save_passenger_trip_allocations(db: &Arc<DBPool>, trips: &Vec<PassengerDirectionInfo>) -> Result<(), rusqlite::Error> {
    let mut db = get_pool(db);
    let tx = db.transaction()?;
    {
        let mut stmt= tx.prepare_cached("REPLACE INTO polar (gtfs, polar, direction) VALUES (?,?,?)")?;
        for trip in trips {
            stmt.execute(params![trip.gtfs, trip.polar, trip.direction])?;
        }
    }
    tx.commit()
}

pub fn query_service(db: &Arc<DBPool>, trip_id: &str) -> rusqlite::Result<ServiceQuery> {
    let db = get_pool(db);
    let mut stmt = db.prepare_cached(
        "SELECT r.route_id as route_id, route_short_name as code, trip_headsign as dest, max_stop_seq as mss FROM trips
                INNER JOIN main.routes r on r.route_id = trips.route_id
                WHERE trip_id=?")?;
    stmt.query_row([trip_id], |row| Ok(ServiceQuery {
        route_id: row.get(0)?,
        code: row.get(1)?,
        dest: row.get(2)?,
        mss: row.get(3)?,
    }))
}

pub struct ServiceQuery {
    pub route_id: String,
    pub code: String,
    pub dest: String,
    pub mss: u64
}

pub fn query_stops(db: &Arc<DBPool>, trip_id: &str) -> rusqlite::Result<Vec<StopsQuery>> {
    let db = get_pool(db);
    let mut stmt = db.prepare_cached(
        "SELECT stops.name, stops.name as display_name, stops.locality, indicator as ind, arrival_time as arr,
                    departure_time as dep, l.name as loc, timepoint as major, drop_off_type as doo, pickup_type as puo,
                    stances.lat as lat, stances.long as long, stop_sequence as seq, stops.locality_name AS full_loc
                FROM stop_times
                    INNER JOIN stances on stances.code = stop_times.stop_id
                    INNER JOIN stops on stops.id = stances.stop
                    INNER JOIN localities l on l.code = stops.locality
                WHERE trip_id=? ORDER BY stop_sequence")?;
    let result = Ok(stmt.query_map([trip_id], |row| Ok(StopsQuery {
        name: row.get(0)?,
        display_name: row.get(1)?,
        locality: row.get(2)?,
        ind: row.get(3).ok(),
        arr: TimeDelta::seconds(row.get(4)?),
        dep: TimeDelta::seconds(row.get(5)?),
        loc: row.get(6).ok(),
        major: row.get(7).unwrap_or(true),
        doo: row.get(8)?,
        puo: row.get(9)?,
        lat: row.get(10)?,
        long: row.get(11)?,
        seq: row.get(12)?,
        full_loc: row.get(13)?,
        status: None
    }))?.filter_map(Result::ok).collect_vec());
    result
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StopsQuery {
    pub name: String,
    pub display_name: String,
    pub locality: String,
    pub ind: Option<String>,
    #[serde(deserialize_with = ":: serde_with :: As :: < DurationSeconds < i64 > > :: deserialize")]
    #[serde(serialize_with="duration_to_fmt")]
    pub arr: Duration,
    #[serde(deserialize_with = ":: serde_with :: As :: < DurationSeconds < i64 > > :: deserialize")]
    #[serde(serialize_with="duration_to_fmt")]
    pub dep: Duration,
    pub loc: Option<String>,
    pub major: bool,
    #[serde(deserialize_with="bool_from_u8")]
    pub puo: bool,
    #[serde(deserialize_with="bool_from_u8")]
    pub doo: bool,
    pub long: f64,
    pub lat: f64,
    pub seq: u64,
    #[serde(skip_serializing)]
    pub full_loc: String,
    pub status: Option<String>
}

impl StopsQuery {
    pub(crate) fn position(&self) -> Coord {
        coord! {x: self.lat, y: self.long }
    }
}

fn bool_from_u8<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(de::Unexpected::Unsigned(other as u64), &"0/1"))
    }
}

fn duration_to_fmt<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serializer.serialize_str(format!("{}", (DateTime::<Utc>::default() + *duration).format("%H:%M")).as_str())
}

pub fn query_service_operator(db: &Arc<DBPool>, trip_id: &str) -> rusqlite::Result<OperatorsQuery> {
    let db = get_pool(db);
    let result = db.prepare_cached(
        "SELECT a.agency_id as id, agency_name as name, COALESCE(website, agency_url) as url FROM trips
                 INNER JOIN main.routes r on r.route_id = trips.route_id
                 INNER JOIN main.agency a on r.agency_id = a.agency_id
                 LEFT OUTER JOIN main.traveline t on a.agency_id = t.agency_id
             WHERE trip_id = ?").unwrap()
        .query_row([trip_id], |row| Ok(OperatorsQuery {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
        }));
    result
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OperatorsQuery {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    pub url: String
}

pub fn get_service_shape(db: &Arc<DBPool>, trip_id: &str) -> rusqlite::Result<String> {
    let db = get_pool(db);
    let result = db.prepare_cached("SELECT polyline FROM shapes INNER JOIN trips t on shapes.shape_id = t.shape_id WHERE trip_id=?").unwrap()
        .query_row([trip_id], |row| row.get(0));
    result
}

pub fn get_stop_positions(db: &Arc<DBPool>, current_stop: u32, trip_id: &str) -> Vec<StopPosition> {
    let db = get_pool(db);
    let result = db.prepare_cached("SELECT stop_sequence,long,lat FROM stop_times
        INNER JOIN stances on stances.code = stop_times.stop_id
        WHERE (stop_sequence=?1 - 1 OR stop_sequence=?1) AND trip_id=?2 ORDER BY stop_sequence").unwrap()
        .query_map(params![current_stop, trip_id], |row| Ok(StopPosition {
            stop_sequence: row.get(0)?,
            pos: coord! {x: row.get(1)?, y: row.get(2)? }
        })).unwrap().filter_map(Result::ok).collect_vec();
    result
}

pub struct StopPosition {
    pub stop_sequence: usize,
    pub pos: Coord<f64>
}

pub fn get_stop_info(db: &Arc<DBPool>, name: &str, locality: &str) -> rusqlite::Result<StopInfoQuery> {
    let db = get_pool(db);
    let result = db.prepare_cached("SELECT id, name, locality_name, locality as locality_code FROM stops WHERE name=? AND locality=?")?
        .query_row([name, locality], |row| Ok(StopInfoQuery {
            id: row.get(0)?,
            name: row.get(1)?,
            locality_name: row.get(2)?,
            locality_code: row.get(3)?,
        }));
    result.inspect_err(|e| println!("{e}"))
}

#[derive(Serialize)]
pub struct StopInfoQuery {
    pub id: u64,
    pub name: String,
    pub locality_name: String,
    pub locality_code: String
}

pub fn get_stance_info(db: &Arc<DBPool>, id: u64) -> rusqlite::Result<Vec<StanceInfo>> {
    let db = get_pool(db);
    let result = db.prepare_cached("SELECT code, indicator, street, crs, lat, long FROM stances WHERE stop=?")?
        .query_map([id], |row| Ok(StanceInfo {
            code: row.get(0)?,
            indicator: row.get(1).ok(),
            street: row.get(2).ok(),
            crs: row.get(3).ok(),
            lat: row.get(4)?,
            long: row.get(5)?,
        }))?.filter_map(Result::ok).collect_vec();
    Ok(result)
}

pub fn get_services_between(db: &Arc<DBPool>, from: &DateTime<Utc>, to: &DateTime<Utc>, stop: u64, filter: bool, filter_name: Option<&String>, filter_loc: Option<&String>) -> rusqlite::Result<Vec<StopService>> {
    if from.day() != to.day() {
        let mut day0 = _get_services_between(db, &(*from - TimeDelta::days(1)), from, to, stop, filter, filter_name, filter_loc)?;
        let mut day1 = _get_services_between(db, from, from, to, stop, filter, filter_name, filter_loc)?;
        let mut day2 =  _get_services_between(db, to, from, to, stop, filter, filter_name, filter_loc)?;
        day0.append(&mut day1);
        day0.append(&mut day2);
        Ok(day0)
    } else {
        let mut day0 = _get_services_between(db, &(*from - TimeDelta::days(1)), from, to, stop, filter, filter_name, filter_loc)?;
        let mut day1 = _get_services_between(db, from, from, to, stop, filter, filter_name, filter_loc)?;
        day0.append(&mut day1);
        Ok(day0)
    }
}

fn _get_services_between(db: &Arc<DBPool>, day: &DateTime<Utc>, from: &DateTime<Utc>, to: &DateTime<Utc>, stop: u64, filter: bool, filter_name: Option<&String>, filter_loc: Option<&String>) -> rusqlite::Result<Vec<StopService>> {
    let day_date = zero_time(day);
    let from_num = (*from - day_date).num_seconds().max(0);
    let to_num = (*to - day_date).num_seconds();
    if to_num < 0 {
        return Ok(Vec::new());
    }
    
    let db = get_pool(db);
    let result = db.prepare_cached("SELECT stop_times.trip_id,coalesce(stop_headsign,t.trip_headsign,'') as trip_headsign, departure_time,
                    s.indicator,r.route_short_name,a.agency_id as operator_id,a.agency_name as operator_name,stop_sequence as seq
                FROM stop_times
                    INNER JOIN trips t on stop_times.trip_id = t.trip_id
                    INNER JOIN stances s ON stop_times.stop_id = s.code
                    INNER JOIN routes r on r.route_id = t.route_id
                    INNER JOIN main.agency a on r.agency_id = a.agency_id
                    LEFT OUTER JOIN main.calendar c on t.service_id = c.service_id
                    LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=:date)
                WHERE
                    s.stop=?1 AND
                    stop_times.stop_sequence <> t.max_stop_seq AND
                    departure_time IS NOT NULL
                    AND ((start_date <= ?2 AND end_date >= ?2 AND (validity & (1 << ?3)) <> 0) OR exception_type=1)
                    AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                    AND departure_time >= ?4 AND departure_time <= ?5
                    AND pickup_type <> 1
                    AND (?6 <> 1 OR EXISTS (SELECT stop_sequence AS inner_seq FROM stop_times WHERE trip_id=t.trip_id AND inner_seq > seq AND stop_id IN (SELECT code FROM stances WHERE stop=(SELECT id FROM stops WHERE locality=?8 AND name=?7))))
                ORDER BY departure_time")?
        .query_map(params![
            stop, u64::from_str(day.format("%Y%m%d").to_string().as_str()).unwrap(), day.weekday().num_days_from_monday(),
            from_num, to_num, u64::from(filter), filter_name, filter_loc
        ], |row| Ok(StopService {
            trip_id: row.get(0)?,
            trip_headsign: row.get(1)?,
            departure_time: vec![day_date + TimeDelta::seconds(row.get::<_, i64>(2)?)],
            indicator: row.get::<_, String>(3).map(|ind| vec![ind]).unwrap_or(vec![]),
            route_short_name: row.get(4)?,
            operator_id: row.get(5)?,
            operator_name: row.get(6)?,
            stop_sequence: row.get(7)?,
            _type: "bus".to_string(),
            colour: "#777".to_string(),
            status: None
        }))?.filter_map(Result::ok).collect_vec();
    Ok(result)
}

#[derive(Serialize)]
pub struct StanceInfo {
    pub code: String,
    pub indicator: Option<String>,
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
    pub lat: f64,
    pub long: f64
}

#[serde_nested]
#[derive(Serialize, PartialEq, PartialOrd)]
pub struct StopService {
    pub trip_id: String,
    pub trip_headsign: String,
    #[serde_nested(sub="DateTime<Utc>", serde(serialize_with = "serialize_as_hhmm"))]
    pub departure_time: Vec<DateTime<Utc>>,
    pub indicator: Vec<String>,
    pub route_short_name: String,
    pub operator_id: String,
    pub operator_name: String,
    #[serde(rename = "seq")]
    pub stop_sequence: u64,
    #[serde(rename = "type")]
    pub _type: String,
    pub colour: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>
}

fn serialize_as_hhmm<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    serializer.serialize_str(format!("{}", date.format("%H:%M")).as_str())
}