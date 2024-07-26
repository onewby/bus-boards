use std::{fmt, fs};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, DurationRound, TimeDelta, TimeZone, Utc};
use chrono_tz::Europe::London;
use chrono_tz::OffsetComponents;
use futures::future::BoxFuture;
use futures::FutureExt;
use geo::{Closest, GeodesicDistance, HaversineClosestPoint};
use geo_types::{CoordNum, Line, Point};
use memoize::memoize;
use reqwest::{IntoUrl, StatusCode};
use serde::de;
use serde::de::{SeqAccess, Visitor};
use tokio::time::sleep;
use rand::{random, Rng, thread_rng};

use crate::util::URLParseError::{DownloadError, ParsingError, StatusCodeError};

pub fn zero_day(date: &DateTime<Utc>) -> DateTime<Utc> {
    date.sub(Duration::from_millis(date.duration_trunc(TimeDelta::days(1)).unwrap().timestamp_millis() as u64))
}

pub fn zero_time(date: &DateTime<Utc>) -> DateTime<Utc> {
    date.duration_trunc(TimeDelta::days(1)).unwrap()
}

pub enum URLParseError {
    DownloadError(reqwest::Error),
    StatusCodeError(StatusCode),
    ParsingError(reqwest::Error)
}

impl Display for URLParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return match self {
            DownloadError(err) => {f.write_fmt(format_args!("Download error: {err}"))}
            StatusCodeError(code) => {f.write_fmt(format_args!("Status code not success: {code}"))}
            ParsingError(err) => {f.write_fmt(format_args!("Parsing error: {err}"))}
        }
    }
}

pub fn get_url_with_retries<'a, T: Send, U: IntoUrl + Send + 'a, Fn: Future<Output=reqwest::Result<T>>+Sized + 'a + Send>(url: U, parser: fn(reqwest::Response) -> Fn, retries: u64) -> BoxFuture<'a, Result<T, URLParseError>> {
    async move {
        match get_url::<T, _, _>(url.as_str(), parser).await {
            Ok(result) => Ok(result),
            Err(e) => {
                if retries <= 0 {
                    Err(e)
                } else {
                    let random_wait = { thread_rng().gen_range(1000..2000) };
                    sleep(Duration::from_millis(random_wait)).await;
                    get_url_with_retries(url, parser, retries - 1).await
                }
            }
        }
    }.boxed()
}

pub async fn get_url<T, U: IntoUrl, Fn: Future<Output=reqwest::Result<T>>+Sized>(url: U, parser: fn(reqwest::Response) -> Fn) -> Result<T, URLParseError> {
    match reqwest::get(url).await {
        Ok(config_resp) => {
            if config_resp.status().is_success() {
                parser(config_resp).await.map_err(ParsingError)
            } else {
                Err(StatusCodeError(config_resp.status()))
            }
        }
        Err(e) => Err(DownloadError(e))
    }
}

pub fn gtfs_time(time: &DateTime<Utc>) -> String {
    return time.format("%H:%M:%S").to_string();
}

pub fn gtfs_date(date: &DateTime<Utc>) -> String {
    return date.format("%Y%m%d").to_string();
}

pub fn relative_to(date: &DateTime<Utc>, time: &DateTime<Utc>) -> DateTime<Utc> {
    DateTime::from_timestamp(time.timestamp() - date.duration_trunc(TimeDelta::days(1)).unwrap().timestamp(), 0).unwrap()
}

pub fn f64_cmp(x: &f64, y: &f64) -> Ordering {
    x.partial_cmp(y).unwrap()
}

struct PointVisitor<T: CoordNum = f64> {
    marker: PhantomData<fn() -> Point<T>>
}

impl<T: CoordNum> PointVisitor<T> {
    fn new() -> Self {
        PointVisitor {
            marker: PhantomData
        }
    }
}

impl<'de, T: CoordNum + serde::Deserialize<'de>> Visitor<'de> for PointVisitor<T> {
    type Value = Point<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("[lon,lat]")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error> where V: SeqAccess<'de> {
        let x = seq.next_element()?.ok_or_else(|| de::Error::missing_field("x"))?;
        let y = seq.next_element()?.ok_or_else(|| de::Error::missing_field("y"))?;
        Ok(Point::<T>::new(x, y))
    }
}

pub fn deserialize_point_array<'de, D>(deserializer: D) -> Result<Point<f64>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    deserializer.deserialize_seq(PointVisitor::<f64>::new())
}

pub fn load_last_update(file: &str) -> DateTime<Utc> {
    fs::read_to_string(file).ok().and_then(|s| DateTime::<Utc>::from_str(s.as_str()).ok())
        .unwrap_or(DateTime::from_timestamp_millis(0).unwrap())
}

pub fn save_last_update(file: &str, time: &DateTime<Utc>) {
    fs::write(file, time.to_string()).expect("Could not write last update time to file!");
}

pub fn adjust_timestamp(time: &DateTime<Utc>) -> DateTime<Utc> {
    let offset = get_bst_offset();
    time.add(offset)
}

#[memoize(TimeToLive: Duration::from_secs(3600))]
pub fn get_bst_offset() -> chrono::Duration {
    London.offset_from_utc_datetime(&Utc::now().naive_utc()).dst_offset()
}

pub fn haversine_closest_point(s: &Line<f64>, loc: &Point<f64>) -> Point<f64> {
    match s.haversine_closest_point(loc) {
        Closest::Intersection(point) => point,
        Closest::SinglePoint(point) => point,
        Closest::Indeterminate => Point(s.start)
    }
}

pub fn get_geo_linepoint_distance(s: &Line<f64>, loc: &Point<f64>) -> f64 {
    loc.geodesic_distance(&haversine_closest_point(s, loc))
}