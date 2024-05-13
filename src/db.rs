use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use chrono::{Datelike, DateTime, DurationRound, TimeDelta, Utc};
use geo_types::Point;
use itertools::Itertools;
use memoize::memoize;
use prost::Message;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, Params, params};
use rusqlite::types::Value;
use crate::bus_prediction::TripCandidate;
use crate::config::BBConfig;
use crate::passenger::PassengerDirectionInfo;
use crate::util::{gtfs_date, relative_to, zero_day};

pub type DBPool = Pool<SqliteConnectionManager>;
pub type PooledConn = PooledConnection<SqliteConnectionManager>;

pub fn open_db() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file("/Users/onewby/Documents/BetterBuses/bus-site/stops.sqlite")
        .with_init(|s| rusqlite::vtab::array::load_module(s));
    Pool::new(manager).unwrap()
}

pub fn get_pool(db: &Arc<DBPool>) -> PooledConn {
    let mut conn: Result<PooledConnection<SqliteConnectionManager>, r2d2::Error>;
    while {
        conn = db.get();
        conn.is_err()
    } {}
    conn.unwrap()
}

pub fn get_string<P: Params>(db: &Arc<DBPool>, query: &str, params: P) -> Result<String, String> {
    get_pool(db).prepare_cached(query).unwrap().query_row(params, |aid| aid.get(0)).map_err(|e| e.to_string())
}

#[memoize(Ignore: db)]
pub fn get_agency(db: &Arc<DBPool>, code: String) -> Result<String, String> {
    get_string(db, "SELECT agency_id FROM traveline WHERE code=?", params![code])
}

#[memoize(Ignore: db)]
pub fn get_route(db: &Arc<DBPool>, code: String, route: String) -> Result<String, String> {
    get_string(db, "SELECT route_id FROM routes INNER JOIN main.traveline t on routes.agency_id = t.agency_id WHERE code=? AND route_short_name=?", (code, route))
}

#[memoize(Ignore: db)]
pub fn get_route_id(db: &Arc<DBPool>, agency_id: String, route: String) -> Result<String, String> {
    get_string(db, "SELECT route_id FROM routes WHERE agency_id=? AND upper(route_short_name)=upper(?)", (agency_id, route))
}

fn trip_query(query: &str, db: &Arc<DBPool>, date: &DateTime<Utc>, start_before: i64, end_after: i64, specifier: &str) -> Vec<TripCandidate>  {
    let date_secs = zero_day(date).timestamp();

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

#[memoize(Ignore: db)]
pub fn get_line_segments(db: &Arc<DBPool>, route_id: String) -> HashMap<String, Point> {
    HashMap::from_iter(get_pool(db).prepare_cached(
        r#"SELECT DISTINCT code as stop_id, lat as y, long as x FROM stances
                WHERE code IN (
                   SELECT stop_id FROM stop_times
            INNER JOIN main.trips t on t.trip_id = stop_times.trip_id WHERE t.route_id=?)"#).unwrap()
        .query_map(params![route_id], |result| Ok((result.get("stop_id")?, Point::new(result.get("x")?, result.get("y")?)))).unwrap().filter_map(|v| v.ok()))
}

#[derive(Clone)]
pub struct LothianDBPattern {
    pub(crate) route: String,
    pub(crate) pattern: String
}

pub fn get_lothian_patterns_tuples(db: &Arc<DBPool>) -> Vec<LothianDBPattern> {
    get_pool(db).prepare_cached(r#"SELECT * FROM lothian"#).unwrap()
        .query_map(params![], |row| Ok(LothianDBPattern {route: row.get("route")?, pattern: row.get("pattern")? })).unwrap()
        .filter_map(|x: Result<LothianDBPattern, _>| x.ok()).collect()
}

#[memoize(Ignore: db)]
pub fn get_lothian_route(db: &Arc<DBPool>, route: String) -> Option<String> {
    get_pool(db).prepare_cached("SELECT route_id FROM routes WHERE route_short_name=? AND agency_id IN (\"OP596\", \"OP597\", \"OP598\")").unwrap()
        .query_row(params![route], |row| row.get("route_id")).ok()
}

#[derive(Clone)]
pub struct StagecoachRoute {
    pub trip_id: String,
    pub route_id: String,
    pub stop_seq: u64,
    pub stop_id: String
}

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
        ":dep_time": zero_day(departure).duration_trunc(TimeDelta::minutes(1)).unwrap().timestamp(),
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

#[derive(Clone)]
pub struct CoachRoute {
    pub agency_id: String,
    pub route_id: String,
    pub route_short_name: String
}

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

pub struct CoachTrip {
    pub trip_id: String,
    pub route: Vec<String>,
    pub times: Vec<usize>,
    pub seqs: Vec<usize>,
    pub std_loc: String,
    pub sta_loc: String
}

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
            ":startTime": zero_day(start).timestamp(),
            ":endTime": relative_to(start, end).timestamp(),
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

pub fn reset_lothian(db: &Arc<DBPool>) {
    get_pool(db).execute("DELETE FROM polar WHERE direction IS NULL", params![]).unwrap();
}

pub type RouteID = String;
pub type RouteName = String;
pub fn get_operator_routes(db: &Arc<DBPool>, agency_id: &str) -> Vec<(RouteID, RouteName)> {
    get_pool(db).prepare_cached("SELECT route_id,route_short_name FROM routes WHERE agency_id=?").unwrap()
        .query_map(params![agency_id], |row| Ok((row.get("route_id")?, row.get("route_short_name")?)))
        .unwrap().filter_map(|c| c.ok()).collect_vec()
}

pub struct LothianGTFSTrip {
    pub min_stop_time: i64,
    pub max_stop_time: i64,
    pub origin_stop: String,
    pub dest_stop: String,
    pub trip_id: String
}

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

pub fn save_lothian_pattern_allocations(db: &Arc<DBPool>, pattern: &str, trip_ids: &[String]) -> Result<(), rusqlite::Error> {
    let mut db = get_pool(db);
    let tx = db.transaction()?;
    {
        let mut stmt= tx.prepare_cached("INSERT INTO polar (gtfs, polar) VALUES (?,?)")?;
        for trip_id in trip_ids {
            stmt.execute(params![trip_id, pattern])?;
        }
    }
    tx.commit()
}

pub fn reset_passenger(db: &Arc<DBPool>) {
    get_pool(db).execute("DELETE FROM polar WHERE direction IS NOT NULL", params![]).unwrap();
}

pub struct PassengerRouteTrip {
    pub min_stop_time: i64,
    pub max_stop_time: i64,
    pub trip_id: String
}

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

pub fn save_passenger_trip_allocations(db: &Arc<DBPool>, trips: &Vec<PassengerDirectionInfo>) -> Result<(), rusqlite::Error> {
    let mut db = get_pool(db);
    let tx = db.transaction()?;
    {
        let mut stmt= tx.prepare_cached("INSERT INTO polar (gtfs, polar, direction) VALUES (?,?,?)")?;
        for trip in trips {
            stmt.execute(params![trip.gtfs, trip.polar, trip.direction])?;
        }
    }
    tx.commit()
}