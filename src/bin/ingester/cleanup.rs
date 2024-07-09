use rusqlite::Connection;
use std::error::Error;
use BusBoardsServer::config::{LastUpdates, save_updates};
use crate::{Localities, Stance};

fn clean_arrivals(db: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Cleaning up arrivals");
    let localities: Localities = crate::load_localities_json();
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

    let arrivals: Vec<ArrivalsSelectResult> = {
        let mut stmt = db.prepare(select_all.as_str())?;
        let x: Vec<ArrivalsSelectResult> = stmt.query_map([], |row| {
            Ok(ArrivalsSelectResult {
                arr_id: row.get(0)?,
                arr_time: row.get(1)?,
                dep_id: row.get(2)?,
            })
        })?.filter_map(|x| x.ok()).collect();
        x
    };

    {
        let tx = db.transaction()?;
        for arrival in arrivals {
            tx.prepare_cached("UPDATE stop_times SET arrival_time=? WHERE rowid=?")?.execute([arrival.arr_time, arrival.dep_id])?;
            tx.prepare_cached("DELETE FROM stop_times WHERE rowid=?")?.execute([arrival.arr_id])?;
        }
        tx.commit()?;
    }
    Ok(())
}

struct ArrivalsSelectResult {
    arr_id: i32,
    arr_time: i32,
    dep_id: i32
}

fn clean_stops(conn: &Connection) -> Result<(), rusqlite::Error> {
    println!("Cleaning up stops");
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    conn.execute("DELETE FROM stops WHERE stops.id NOT IN (SELECT DISTINCT stances.stop FROM stop_times INNER JOIN stances ON stances.code=stop_id) AND (NOT EXISTS(SELECT 1 FROM stances WHERE stances.stop=stops.id AND crs IS NOT NULL));", [])?;
    println!("Rebuilding stops_search");
    // Rebuild stops_search table
    conn.execute("DROP TABLE IF EXISTS stops_search;", [])?;
    conn.execute("CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED, locality UNINDEXED);", [])?;
    conn.execute("INSERT INTO stops_search(name, parent, qualifier, id, locality) SELECT stops.name, stops.locality_name, qualifier, stops.id, stops.locality FROM stops INNER JOIN localities l on l.code = stops.locality;", [])?;
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

pub fn cleanup(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Updating sequence numbers");
    conn.execute("UPDATE trips SET min_stop_seq=(SELECT min(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)", ())?;
    conn.execute("UPDATE trips SET max_stop_seq=(SELECT max(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)", ())?;
    clean_arrivals(conn).expect("Clean arrivals");
    clean_stops(conn).expect("Clean stops");
    reset_polar().expect("Reset Polar");
    patch_display_names(conn).expect("Display name patching");
    remove_traveline_ember(conn).expect("Patch Ember");
    Ok(())
}
