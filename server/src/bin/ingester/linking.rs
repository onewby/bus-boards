use std::error::Error;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use chrono::{Utc};
use itertools::Itertools;
use polars::export::rayon::current_num_threads;
use polars::export::rayon::iter::*;
use r2d2::{ManageConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rand::Rng;
use rusqlite::{params, CachedStatement};
use rusqlite::types::Value;

pub fn link_trips(db_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Linking trips");
    
    let db_pool = Pool::builder()
        .max_size(current_num_threads() as u32 * 2)
        .connection_timeout(Duration::from_secs(60))
        .build(SqliteConnectionManager::file(db_path)
            .with_init(|s| rusqlite::vtab::array::load_module(s)))?;

    println!("Running database optimise");
    get_pool(&db_pool).execute("pragma optimize", []).unwrap();

    println!("Finding routes");
    let routes = get_pool(&db_pool).prepare("SELECT route_id, origin.stop_id, postorigin_stance.stop, predest_stance.stop, dest.stop_id, group_concat(trips.trip_id,',')
        FROM trips
            INNER JOIN stop_times AS origin ON origin.trip_id=trips.trip_id AND origin.stop_sequence=trips.min_stop_seq
            INNER JOIN stop_times AS postorigin ON postorigin.trip_id=trips.trip_id AND postorigin.stop_sequence=trips.min_stop_seq+1
            INNER JOIN stop_times AS predest ON predest.trip_id=trips.trip_id AND predest.stop_sequence=trips.max_stop_seq-1
            INNER JOIN stop_times AS dest ON dest.trip_id=trips.trip_id AND dest.stop_sequence=trips.max_stop_seq
            INNER JOIN stances AS postorigin_stance ON postorigin.stop_id=postorigin_stance.code
            INNER JOIN stances AS predest_stance ON predest.stop_id=predest_stance.code
        GROUP BY route_id, origin.stop_id, postorigin_stance.stop, predest_stance.stop, dest.stop_id")?
        .query_map([], |row| Ok(RouteInfo {
            route_id: row.get(0)?,
            origin_stop: row.get(1)?,
            postorigin_stance: row.get(2)?,
            predest_stance: row.get(3)?,
            dest_stop: row.get(4)?,
            trips: row.get(5)?,
        }))?
        .filter_map(|o| o.ok())
        .collect_vec();

    println!("Found {} different routes", routes.len());

    let start = Utc::now();

    let groups = routes.iter()
        .group_by(|r| r.route_id.clone())
        .into_iter()
        .map(|(route_id, group)| (route_id, group.collect_vec()))
        .collect_vec();

    let results: Vec<TripsResult> = groups.par_iter()
        .map(|(route_id, group)| {
            let db = get_pool(&db_pool);
            let mut trips_stmt = get_trips_stmt(&db);
            group.iter().map(|r1| {
                let t1_values = Rc::new(r1.trips.split(',').map(|s| Value::from(s.to_string())).collect::<Vec<Value>>());
                let t2_values = Rc::new(group.iter()
                    .filter(|r2| r1.dest_stop == r2.origin_stop && r1.predest_stance != r2.postorigin_stance)
                    .flat_map(|r2| {
                        r2.trips.split(',').map(|s| Value::from(s.to_string()))
                    }).collect_vec());
                trips_stmt.query_map(params![t1_values, t2_values], |row| {
                    Ok(TripsResult {
                        trip1: row.get(0)?,
                        trip2: row.get(1)?,
                        diff: row.get(2)?,
                        show_then: row.get(3)?
                    })
                }).unwrap().filter_map(Result::ok)
                    .group_by(|r| r.trip1.clone())
                    .into_iter()
                    .map(|(t1, group)| {
                        group.min_by_key(|row| row.diff).unwrap()
                    }).collect_vec()
            }).collect_vec()
        }).flatten().flatten().collect();
    
    let end = Utc::now();
    
    println!("Taken {} - now inserting links into database", end - start);
    
    let mut db = get_pool(&db_pool);
    let tx = db.transaction()?;
    {
        let mut stmt = tx.prepare_cached("INSERT INTO links (\"from\", \"to\", show_then) VALUES (?, ?, ?)")?;
        for result in results {
            stmt.execute(params![result.trip1.as_str(), result.trip2.as_str(), &result.show_then])?;
        }
    }
    tx.commit()?;
    
    Ok(())
}

/// Get connection from database pool
pub fn get_pool(db: &Pool<SqliteConnectionManager>) -> PooledConnection<SqliteConnectionManager> {
    let mut conn: Result<PooledConnection<SqliteConnectionManager>, r2d2::Error>;
    while {
        conn = db.get();
        if conn.is_err() {
            sleep(Duration::from_millis(rand::thread_rng().gen_range(500..1500)))
        }
        conn.is_err()
    } {}
    conn.unwrap()
}

pub fn get_trips_stmt(db: &PooledConnection<SqliteConnectionManager>) -> CachedStatement<'_> {
    db.prepare_cached(r#"
            SELECT t1id, t2id, dep_diff, (t2_dest_lstop.locality NOT IN
               (SELECT locality FROM stop_times
                                         INNER JOIN stances ON stances.code=stop_times.stop_id
                                         INNER JOIN stops ON stops.id=stances.stop
                WHERE stop_times.trip_id=t1id)) AS show_then
            FROM (SELECT ROW_NUMBER() OVER (PARTITION BY t1id ORDER BY dep_diff) as RowNum, *
                  FROM (SELECT t1.trip_id                                                    AS t1id,
                               t2.trip_id                                                    AS t2id,
                               t2.max_stop_seq                                               AS t2mss,
                               (t2_origin_stop.departure_time - t1_dest_stop.departure_time) AS dep_diff
                        FROM trips AS t1
                             CROSS JOIN trips as t2
                             INNER JOIN stop_times AS t1_dest_stop
                                        ON t1_dest_stop.trip_id = t1.trip_id AND t1.max_stop_seq = t1_dest_stop.stop_sequence
                             INNER JOIN stop_times AS t2_origin_stop ON t2_origin_stop.trip_id = t2.trip_id AND
                                                                        t2.min_stop_seq = t2_origin_stop.stop_sequence
                             LEFT OUTER JOIN calendar AS t1_calendar ON t1_calendar.service_id = t1.service_id
                             LEFT OUTER JOIN calendar AS t2_calendar ON t2_calendar.service_id = t2.service_id AND (t1_calendar.validity & t2_calendar.validity) <> 0 AND (
                                 (t1_calendar.start_date <= t2_calendar.start_date AND t1_calendar.end_date >= t2_calendar.start_date)
                                     OR (t1_calendar.start_date > t2_calendar.start_date AND t2_calendar.end_date >= t1_calendar.start_date))
                        WHERE t1.trip_id IN rarray(?1)
                          AND t2.trip_id IN rarray(?2)
                          AND (t2_calendar.validity IS NOT NULL
                              OR EXISTS(SELECT * FROM
                                (SELECT * FROM calendar_dates WHERE service_id=t1.service_id AND exception_type=1
                                 UNION SELECT * FROM calendar_dates WHERE service_id=t2.service_id AND exception_type=1)
                                         GROUP BY date HAVING count(*) >= 2))
                          AND dep_diff BETWEEN 0 AND 600)) X
                INNER JOIN stop_times AS t2_dest_stop
                        ON t2_dest_stop.trip_id = t2id AND t2mss = t2_dest_stop.stop_sequence
                INNER JOIN stances AS t2_dest_stance ON t2_dest_stance.code = t2_dest_stop.stop_id
                INNER JOIN stops AS t2_dest_lstop ON t2_dest_lstop.id = t2_dest_stance.stop
            WHERE RowNum = 1
        "#).unwrap()
}

struct TripsResult {
    trip1: String,
    trip2: String,
    diff: u64,
    show_then: u8
}

pub struct RouteInfo {
    route_id: String,
    origin_stop: String,
    postorigin_stance: u64,
    predest_stance: u64,
    dest_stop: String,
    trips: String
}