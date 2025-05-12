use std::error::Error;
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
use thread_local::ThreadLocal;

pub fn link_trips(db_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Linking trips");
    
    let db_pool = Pool::builder()
        .max_size(current_num_threads() as u32 * 2)
        .connection_timeout(Duration::from_secs(60))
        .build(SqliteConnectionManager::file(db_path)
            .with_init(|s| rusqlite::vtab::array::load_module(s)))?;

    println!("Running database optimise");
    get_pool(&db_pool).execute("pragma optimize", []).unwrap();
    get_pool(&db_pool).execute_batch("ANALYZE trips; ANALYZE stop_times; ANALYZE stances; ANALYZE trips_route_id_index; ANALYZE stop_times_trip_id_stop_sequence_index;").unwrap();

    println!("Linking using block ID");
    get_pool(&db_pool).execute(r#"INSERT INTO links SELECT * FROM (SELECT lag(trips.trip_id) over (PARTITION BY block_id ORDER BY departure_time) AS "from", trips.trip_id AS "to", 0 FROM trips
         INNER JOIN stop_times AS departure ON departure.trip_id=trips.trip_id AND departure.stop_sequence=(SELECT max(stop_sequence) FROM stop_times WHERE trip_id=trips.trip_id)
         WHERE block_id IS NOT NULL)
         WHERE "from" IS NOT NULL"#, [])?;
    
    println!("Finding routes to link manually");
    let routes = get_pool(&db_pool).prepare(
        "SELECT route_id AS rid, service_id
                   FROM trips
                   WHERE route_id NOT IN (SELECT route_id FROM trips WHERE block_id IS NOT NULL)
                   GROUP BY rid, service_id
                   HAVING count(trips.trip_id) > 1")?
        .query_map([], |row| Ok(RouteInfo {
            route_id: row.get(0)?,
            service_id: row.get(1)?
        }))?
        .filter_map(Result::ok)
        .collect_vec();

    println!("Found {} different routes", routes.len());

    let start = Utc::now();
    
    let results: Vec<TripsResult> = {
        let tl_conn = ThreadLocal::new();
        routes.par_iter()
            .map(|route| {
                let conn = tl_conn.get_or(|| get_pool(&db_pool));
                let mut trips_stmt = get_trips_stmt(conn);
                trips_stmt.query_map(params![route.route_id, route.service_id], |row| Ok(TripsResult {
                    trip1: row.get(0)?,
                    trip2: row.get(1)?,
                    diff: row.get(2)?,
                    show_then: row.get(3)?,
                })).unwrap().filter_map(Result::ok).collect_vec()
            }).flatten().collect()
    };
    
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
            eprintln!("note: db pool get fail, retrying");
            sleep(Duration::from_millis(rand::thread_rng().gen_range(500..1500)))
        }
        conn.is_err()
    } {}
    conn.unwrap()
}

pub fn get_trips_stmt(db: &PooledConnection<SqliteConnectionManager>) -> CachedStatement<'_> {
    db.prepare_cached(r#"
            SELECT t1.trip_id AS t1id, t2.trip_id AS t2id, min(t2_origin.departure_time - t1_dest.arrival_time) AS dep_diff,
              (t2_dest_stop.locality NOT IN
               (SELECT locality FROM stop_times
                                         INNER JOIN stances ON stances.code=stop_times.stop_id
                                         INNER JOIN stops ON stops.id=stances.stop
                WHERE stop_times.trip_id=t1.trip_id)) AS show_then
            FROM trips AS t1
              INNER JOIN trips AS t2
              INNER JOIN stop_times AS t1_dest ON t1_dest.trip_id=t1.trip_id AND t1_dest.stop_sequence=(SELECT max(stop_sequence) FROM stop_times WHERE trip_id=t1.trip_id)
              INNER JOIN stop_times AS t1_predest ON t1_predest.trip_id=t1.trip_id AND t1_predest.stop_sequence=t1_dest.stop_sequence-1
              INNER JOIN stop_times AS t2_origin ON t2_origin.trip_id=t2.trip_id AND t2_origin.stop_sequence=(SELECT min(stop_sequence) FROM stop_times WHERE trip_id=t2.trip_id)
              INNER JOIN stop_times AS t2_postorigin ON t2_postorigin.trip_id=t2.trip_id AND t2_postorigin.stop_sequence=t2_origin.stop_sequence+1
              INNER JOIN stop_times AS t2_dest ON t2_dest.trip_id=t2.trip_id AND t2_dest.stop_sequence=(SELECT max(stop_sequence) FROM stop_times WHERE trip_id=t2.trip_id)
              INNER JOIN stances AS t2_dest_stance ON t2_dest_stance.code = t2_dest.stop_id
              INNER JOIN stops AS t2_dest_stop ON t2_dest_stance.stop = t2_dest_stop.id
              INNER JOIN stances AS postorigin_stance ON t2_postorigin.stop_id=postorigin_stance.code
              INNER JOIN stances AS predest_stance ON t1_predest.stop_id=predest_stance.code
        WHERE t1.trip_id <> t2.trip_id
            AND t1.route_id=?1 AND t1.service_id=?2
            AND t1.route_id=t2.route_id AND t1.service_id=t2.service_id
            AND t1.direction_id <> t2.direction_id
            AND t1_dest.stop_id=t2_origin.stop_id
            AND postorigin_stance.stop <> predest_stance.stop
            AND t2_origin.departure_time - t1_dest.arrival_time BETWEEN 0 AND 600
        GROUP BY t1.trip_id
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
    service_id: String
}