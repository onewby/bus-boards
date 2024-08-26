use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use chrono::{Utc};
use itertools::Itertools;
use r2d2::{ManageConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rand::Rng;
use rusqlite::params;

pub fn link_trips(db_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Linking trips - expected to take ~11mins");
    
    let db_pool = Pool::builder()
        .max_size(16)
        .connection_timeout(Duration::from_secs(60))
        .build(SqliteConnectionManager::file(db_path)
            .with_init(|s| rusqlite::vtab::array::load_module(s)))?;
    
    let start = Utc::now();

    let db = db_pool.get().unwrap();
    let mut trips_stmt = db.prepare_cached(r#"
            SELECT t1.trip_id AS t1id, t2.trip_id AS t2id,
                   (t2_origin_stop.departure_time - t1_dest_stop.departure_time) AS dep_diff,
                   (t1_origin_stance.stop<>t2_dest_stance.stop) AS show_then
            FROM trips AS t1
                CROSS JOIN trips as t2
                INNER JOIN stop_times AS t1_origin_stop ON t1_origin_stop.trip_id=t1.trip_id AND t1.min_stop_seq=t1_origin_stop.stop_sequence
                INNER JOIN stop_times AS t1_dest_stop ON t1_dest_stop.trip_id=t1.trip_id AND t1.max_stop_seq=t1_dest_stop.stop_sequence
                INNER JOIN stop_times AS t2_origin_stop ON t2_origin_stop.trip_id=t2.trip_id AND t2.min_stop_seq=t2_origin_stop.stop_sequence
                INNER JOIN stop_times AS t2_dest_stop ON t2_dest_stop.trip_id=t2.trip_id AND t2.max_stop_seq=t2_dest_stop.stop_sequence
                INNER JOIN stances AS t1_origin_stance ON t1_origin_stance.code=t1_origin_stop.stop_id
                INNER JOIN stances AS t2_dest_stance ON t2_dest_stance.code=t2_dest_stop.stop_id
                WHERE t1.route_id=t2.route_id
                    AND t1_dest_stop.stop_id=t2_origin_stop.stop_id
                    AND t1_origin_stop.departure_time < t2_origin_stop.departure_time
                    AND dep_diff <= 120 AND dep_diff >= 0
        "#).unwrap();
    let results = trips_stmt.query_map([], |row| {
        Ok(TripsResult {
            trip1: row.get(0)?,
            trip2: row.get(1)?,
            diff: row.get(2)?,
            show_then: row.get(3)?
        })
    }).unwrap().filter_map(|r| r.ok())
        .group_by(|r| r.trip1.clone())
        .into_iter()
        .map(|(t1, group)| {
            group.min_by_key(|row| row.diff).unwrap()
        })
        .collect_vec();
    
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

struct TripsResult {
    trip1: String,
    trip2: String,
    diff: u64,
    show_then: u8
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