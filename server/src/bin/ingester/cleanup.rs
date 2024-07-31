use rusqlite::Connection;
use std::error::Error;
use std::fs;
use crate::localities::{load_localities_json, Localities, Stance};

fn clean_arrivals(db: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Cleaning up arrivals");
    let localities: Localities = load_localities_json();
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
    conn.execute("CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED, locality UNINDEXED, priority UNINDEXED, station);", [])?;
    conn.execute("INSERT INTO stops_search(name, parent, qualifier, id, locality, priority, station) SELECT stops.name, stops.locality_name, qualifier, stops.id, stops.locality, (SELECT count(*) FROM stop_times INNER JOIN stances ON stop_times.stop_id=stances.code WHERE stances.stop=stops.id),crs FROM stops
                      INNER JOIN localities l on l.code = stops.locality
                      LEFT JOIN (SELECT id,crs FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE crs IS NOT NULL) s ON stops.id=s.id;", [])?;
    Ok(())
}

fn reset_polar() -> Result<(), Box<dyn Error>> {
    let _ = fs::remove_file(".update.lothian");
    let _ = fs::remove_file(".update.passenger");
    Ok(())
}

fn patch_display_names(conn: &mut Connection) -> rusqlite::Result<usize> {
    println!("Patching route display names");
    conn.execute("UPDATE routes SET route_short_name=substr(route_id, 2) WHERE agency_id='Ember'", [])?;
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
    delete_operator(conn, "OP965")
}

fn delete_operator(conn: &mut Connection, agency_id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM stop_times WHERE trip_id=(SELECT trip_id FROM trips INNER JOIN main.routes r on r.route_id = trips.route_id WHERE r.agency_id=?)", [agency_id])?;
    conn.execute("DELETE FROM trips WHERE trip_id=(SELECT trip_id FROM trips INNER JOIN main.routes r on r.route_id = trips.route_id WHERE r.agency_id=?)", [agency_id])?;
    conn.execute("DELETE FROM routes WHERE agency_id=?", [agency_id])?;
    conn.execute("DELETE FROM agency WHERE agency_id=?", [agency_id])
}

fn clean_flix(conn: &mut Connection) -> rusqlite::Result<usize> {
    println!("Cleaning Flix data");
    conn.execute("UPDATE trips SET trip_id=replace(trip_id, '#', '-') WHERE EXISTS (SELECT agency_id FROM routes WHERE routes.route_id=trips.route_id AND agency_id='FLIXBUS-eu')", [])?;
    conn.execute("UPDATE routes SET route_short_name=replace(replace(route_short_name,'UK',''), 'FlixBus ', '') WHERE agency_id='FLIXBUS-eu'", [])?;
    conn.execute(r#"
        DELETE FROM routes WHERE routes.agency_id='FLIXBUS-eu' AND routes.route_id NOT IN (SELECT trips.route_id FROM trips
           INNER JOIN routes r on trips.route_id = r.route_id
           INNER JOIN stop_times origin on trips.trip_id = origin.trip_id AND origin.stop_sequence=trips.min_stop_seq
           INNER JOIN stances origin_stance on origin_stance.code = origin.stop_id
           INNER JOIN stops origin_stop on origin_stop.id = origin_stance.stop
           INNER JOIN stop_times dest on trips.trip_id = dest.trip_id AND dest.stop_sequence=trips.max_stop_seq
           INNER JOIN stances dest_stance on dest_stance.code = dest.stop_id
           INNER JOIN stops dest_stop on dest_stop.id = dest_stance.stop
        WHERE r.agency_id='FLIXBUS-eu' AND (origin_stop.locality<>'Europe' OR dest_stop.locality<>'Europe'));
    "#, [])?;
    conn.execute("UPDATE agency SET agency_name='FlixBus' WHERE agency_id='FLIXBUS-eu'", [])?;
    conn.execute(r#"
    UPDATE trips SET trip_headsign=(
        SELECT CASE WHEN dest_loc_name = 'Europe' THEN dest_name
        WHEN dest_loc_name = 'Centenary Square' THEN 'Birmingham'
        ELSE (SELECT IFNULL(substr(s.locality_name, 0, NULLIF(instr(s.locality_name, '›') - 1, -1)), s.locality_name)
              FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=dest_stop_id) END
        FROM (SELECT trips.trip_id, dest_stop.locality_name AS dest_loc_name, dest_stop.name AS dest_name, dest.stop_id AS dest_stop_id FROM trips
                                INNER JOIN main.stop_times dest on (trips.trip_id = dest.trip_id AND trips.max_stop_seq=dest.stop_sequence)
                                INNER JOIN main.stances st on st.code=dest.stop_id
                                INNER JOIN main.stops dest_stop on dest_stop.id = st.stop
             ) AS trip_subquery
        WHERE trips.trip_id=trip_subquery.trip_id
    ) WHERE (SELECT agency_id FROM routes WHERE trips.route_id=routes.route_id)='FLIXBUS-eu'
    "#, [])?;
    delete_operator(conn, "OP5051")?;
    delete_operator(conn, "FLIXTRAIN-eu")
}

pub fn cleanup(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Updating sequence numbers");
    conn.execute("UPDATE trips SET min_stop_seq=(SELECT min(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)", ())?;
    conn.execute("UPDATE trips SET max_stop_seq=(SELECT max(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)", ())?;
    clean_arrivals(conn).expect("Clean arrivals");
    clean_flix(conn).expect("Clean Flix");
    clean_stops(conn).expect("Clean stops");
    reset_polar().expect("Reset Polar");
    patch_display_names(conn).expect("Display name patching");
    remove_traveline_ember(conn).expect("Patch Ember");
    Ok(())
}
