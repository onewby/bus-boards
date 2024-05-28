use std::error::Error;
use std::str::FromStr;
use csv::ByteRecord;
use pg_interval::Interval;
use postgres::{Client, Statement};
use postgres::types::ToSql;
use serde::{de, Deserialize, Deserializer};
use serde::de::{Unexpected};

pub trait Converter {
    fn get_file(&self) -> &'static str;
    fn get_stmt(&self) -> &'static str;
    fn insert(&self, _: &mut Client, _: &mut Statement, _: &mut ByteRecord) -> Result<(), Box<dyn Error>>;
}

const AGENCY_FILE: &str = "agency.txt";
const AGENCY_STMT: &str = "INSERT INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang, modified) VALUES ($1, $2, $3, $4, $5, NOW())
           ON CONFLICT (agency_id) DO UPDATE SET agency_name=EXCLUDED.agency_name, agency_url=EXCLUDED.agency_url, agency_timezone=EXCLUDED.agency_timezone, agency_lang=EXCLUDED.agency_lang, modified=EXCLUDED.modified";
pub struct AgencyC();

#[derive(Deserialize)]
pub struct Agency<'a> {
    agency_id: &'a str,
    agency_name: &'a str,
    agency_url: &'a str,
    agency_timezone: &'a str,
    agency_lang: &'a str
}

impl Converter for AgencyC {
    fn get_file(&self) -> &'static str { AGENCY_FILE }
    fn get_stmt(&self) -> &'static str { AGENCY_STMT }

    fn insert(&self, db: &mut Client, stmt: &mut Statement, record: &mut ByteRecord) -> Result<(), Box<dyn Error>> {
        let obj: Agency = record.deserialize(None)?;
        let vec: Vec<&(dyn ToSql + Sync)> = vec![&obj.agency_id, &obj.agency_name, &obj.agency_url, &obj.agency_timezone, &obj.agency_lang];
        db.execute(stmt, &vec[..])?;
        Ok(())
    }
}

const ROUTE_FILE: &str = "routes.txt";
const ROUTE_STMT: &str = "INSERT INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type, modified) VALUES ($1, $2, $3, $4, $5, NOW())\
            ON CONFLICT (route_id) DO UPDATE SET agency_id=EXCLUDED.agency_id, route_short_name=EXCLUDED.route_short_name, route_long_name=EXCLUDED.route_long_name, route_type=EXCLUDED.route_type, modified=EXCLUDED.modified";
pub struct RouteC();

#[derive(Deserialize)]
struct Route<'a> {
    route_id: &'a str,
    agency_id: &'a str,
    route_short_name: &'a str,
    route_long_name: &'a str,
    route_type: &'a str
}

impl Converter for RouteC {
    fn get_file(&self) -> &'static str { ROUTE_FILE }
    fn get_stmt(&self) -> &'static str { ROUTE_STMT }

    fn insert(&self, db: &mut Client, stmt: &mut Statement, record: &mut ByteRecord) -> Result<(), Box<dyn Error>> {
        let obj: Route = record.deserialize(None)?;
        let vec: Vec<&(dyn ToSql + Sync)> = vec![&obj.route_id, &obj.agency_id, &obj.route_short_name, &obj.route_long_name, &obj.route_type];
        db.execute(stmt, &vec[..])?;
        Ok(())
    }
}

const CALENDAR_FILE: &str = "calendar.txt";
const CALENDAR_STMT: &str = "INSERT INTO calendar (service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date, modified) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (service_id) DO UPDATE SET monday=EXCLUDED.monday, tuesday=EXCLUDED.tuesday, wednesday=EXCLUDED.wednesday, thursday=EXCLUDED.thursday, friday=EXCLUDED.friday, saturday=EXCLUDED.saturday, sunday=EXCLUDED.sunday, start_date=EXCLUDED.start_date, end_date=EXCLUDED.end_date, modified=EXCLUDED.modified";
pub struct CalendarC();

#[derive(Deserialize)]
struct Calendar<'a> {
    service_id: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    monday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    tuesday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    wednesday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    thursday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    friday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    saturday: bool,
    #[serde(deserialize_with = "bool_from_int")]
    sunday: bool,
    start_date: i32,
    end_date: i32
}

impl Converter for CalendarC {
    fn get_file(&self) -> &'static str { CALENDAR_FILE }
    fn get_stmt(&self) -> &'static str { CALENDAR_STMT }

    fn insert(&self, db: &mut Client, stmt: &mut Statement, record: &mut ByteRecord) -> Result<(), Box<dyn Error>> {
        let obj: Calendar = record.deserialize(None)?;
        let vec: Vec<&(dyn ToSql + Sync)> = vec![&obj.service_id, &obj.monday, &obj.tuesday, &obj.wednesday, &obj.thursday, &obj.friday, &obj.saturday, &obj.sunday, &obj.start_date, &obj.end_date];
        db.execute(stmt, &vec[..])?;
        Ok(())
    }
}

const CALENDARDATE_FILE: &str = "calendar_dates.txt";
const CALENDARDATE_STMT: &str = "INSERT INTO calendar_dates (service_id, date, exception_type, modified) VALUES ($1, $2, $3, NOW())
            ON CONFLICT (service_id, date) DO UPDATE SET exception_type=EXCLUDED.exception_type, modified=EXCLUDED.modified";
pub struct CalendarDateC();

#[derive(Deserialize)]
struct CalendarDate<'a> {
    service_id: &'a str,
    date: i32,
    exception_type: i16
}

impl Converter for CalendarDateC {
    fn get_file(&self) -> &'static str { CALENDARDATE_FILE }
    fn get_stmt(&self) -> &'static str { CALENDARDATE_STMT }

    fn insert(&self, db: &mut Client, stmt: &mut Statement, record: &mut ByteRecord) -> Result<(), Box<dyn Error>> {
        let obj: CalendarDate = record.deserialize(None)?;
        let vec: Vec<&(dyn ToSql + Sync)> = vec![&obj.service_id, &obj.date, &obj.exception_type];
        db.execute(stmt, &vec[..])?;
        Ok(())
    }
}

const TRIP_FILE: &str = "trips.txt";
const TRIP_STMT: &str = "INSERT INTO trips (route_id, service_id, trip_id, trip_headsign, modified) VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (trip_id) DO UPDATE SET route_id=EXCLUDED.route_id,service_id=EXCLUDED.service_id,trip_headsign=EXCLUDED.trip_headsign,modified=EXCLUDED.modified";
pub struct TripC();

#[derive(Deserialize)]
struct Trip<'a> {
    trip_id: &'a str,
    route_id: &'a str,
    service_id: &'a str,
    trip_headsign: &'a str,
    block_id: Option<&'a str>,
    shape_id: Option<&'a str>,
    wheelchair_accessible: Option<&'a str>,
    vehicle_journey_code: &'a str
}

impl Converter for TripC {
    fn get_file(&self) -> &'static str { TRIP_FILE }
    fn get_stmt(&self) -> &'static str { TRIP_STMT }

    fn insert(&self, db: &mut Client, stmt: &mut Statement, record: &mut ByteRecord) -> Result<(), Box<dyn Error>> {
        let obj: Trip = record.deserialize(None)?;
        let vec: Vec<&(dyn ToSql + Sync)> = vec![&obj.trip_id, &obj.route_id, &obj.service_id, &obj.trip_headsign];
        db.execute(stmt, &vec[..])?;
        Ok(())
    }
}

const STOPTIME_FILE: &str = "stop_times.txt";
const STOPTIME_STMT: &str = "INSERT INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, stop_headsign, pickup_type, drop_off_type, timepoint, modified) VALUES ($1, $2, $3, $4, $5, NULLIF($6, ''), $7, $8, $9, NOW())
            ON CONFLICT (trip_id, stop_sequence) DO UPDATE SET arrival_time=EXCLUDED.arrival_time, departure_time=EXCLUDED.departure_time, stop_id=EXCLUDED.stop_id, stop_headsign=EXCLUDED.stop_headsign, pickup_type=EXCLUDED.pickup_type, drop_off_type=EXCLUDED.drop_off_type, timepoint=EXCLUDED.timepoint, modified=EXCLUDED.modified";
pub struct StopTimeC();

#[derive(Deserialize)]
struct StopTime<'a> {
    trip_id: &'a str,
    #[serde(deserialize_with = "interval_from_str")]
    arrival_time: Interval,
    #[serde(deserialize_with = "interval_from_str")]
    departure_time: Interval,
    stop_id: &'a str,
    stop_sequence: i32,
    stop_headsign: &'a str,
    pickup_type: i16,
    drop_off_type: i16,
    shape_dist_traveled: Option<f32>,
    #[serde(deserialize_with = "bool_from_int")]
    timepoint: bool
}

impl Converter for StopTimeC {
    fn get_file(&self) -> &'static str { STOPTIME_FILE }
    fn get_stmt(&self) -> &'static str { STOPTIME_STMT }

    fn insert(&self, db: &mut Client, stmt: &mut Statement, record: &mut ByteRecord) -> Result<(), Box<dyn Error>> {
        let obj: StopTime = record.deserialize(None)?;
        let vec: Vec<&(dyn ToSql + Sync)> = vec![&obj.trip_id, &obj.arrival_time, &obj.departure_time, &obj.stop_id, &obj.stop_sequence, &obj.stop_headsign, &obj.pickup_type, &obj.drop_off_type, &obj.timepoint];
        db.execute(stmt, &vec[..])?;
        Ok(())
    }
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

fn interval_from_str<'de, D>(deserializer: D) -> Result<Interval, D::Error>
    where
        D: Deserializer<'de>,
{
    let iso: &str = Deserialize::deserialize(deserializer)?;
    let parts: Vec<&str> = iso.split(":").collect();
    if parts.len() != 3 { panic!("Invalid format") };
    let hours = i64::from_str(parts[0]).map_err(|_x| de::Error::missing_field(&"hours"))?;
    let mins = i64::from_str(parts[1]).map_err(|_x| de::Error::missing_field(&"minutes"))?;
    let secs = i64::from_str(parts[2]).map_err(|_x| de::Error::missing_field(&"seconds"))?;
    let us = ((((hours * 60) + mins) * 60) + secs) * 1000000;

    Ok(Interval::new(0, 0, us))
}