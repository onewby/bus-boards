use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use chrono::{DateTime, DurationRound, NaiveDateTime, TimeDelta, Utc};
use std::time::Duration;
use std::ops::Sub;
use chrono_tz::Tz::Europe__London;
use geo_types::{CoordNum, Point};
use reqwest::{IntoUrl, StatusCode};
use serde::de;
use serde::de::{MapAccess, SeqAccess, Visitor};
use crate::util::URLParseError::{DownloadError, ParsingError, StatusCodeError};

pub fn zero_day(date: &DateTime<Utc>) -> DateTime<Utc> {
    date.sub(Duration::from_millis(date.duration_trunc(TimeDelta::days(1)).unwrap().timestamp_millis() as u64))
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

pub async fn get_url<T, U: IntoUrl, Fn: Future<Output=reqwest::Result<T>>+Sized>(url: U, parser: fn(reqwest::Response) -> Fn) -> Result<T, URLParseError> {
    return match reqwest::get(url).await {
        Ok(config_resp) => {
            return if config_resp.status().is_success() {
                parser(config_resp).await.map_err(|e| ParsingError(e))
            } else {
                Err(StatusCodeError(config_resp.status()))
            }
        }
        Err(e) => Err(DownloadError(e))
    };
}

pub fn gtfs_time(time: &DateTime<Utc>) -> String {
    return time.format("%H:%M:%S").to_string();
}

pub fn gtfs_date(date: &DateTime<Utc>) -> String {
    return date.format("%Y%m%d").to_string();
}

pub fn relative_to(date: &DateTime<Utc>, time: &DateTime<Utc>) -> DateTime<Utc> {
    return DateTime::from_timestamp(time.timestamp() - date.duration_trunc(TimeDelta::days(1)).unwrap().timestamp(), 0).unwrap();
}

pub fn f64_cmp(x: &f64, y: &f64) -> Ordering {
    x.partial_cmp(y).unwrap()
}

struct FirstVisitor;

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field { Date, Time }

impl<'de> Visitor<'de> for FirstVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("{date: String, time: String}")
    }

    fn visit_map<V>(self, mut map: V) -> Result<DateTime<Utc>, V::Error>
        where
            V: MapAccess<'de>,
    {
        let mut date = None;
        let mut time = None;
        while let Some(key) = map.next_key()? {
            match key {
                Field::Date => {
                    if date.is_some() {
                        return Err(de::Error::duplicate_field("date"));
                    }
                    date = Some(map.next_value()?);
                }
                Field::Time => {
                    if time.is_some() {
                        return Err(de::Error::duplicate_field("time"));
                    }
                    time = Some(map.next_value()?);
                }
            }
        }
        let date: String = date.ok_or_else(|| de::Error::missing_field("date"))?;
        let time: String = time.ok_or_else(|| de::Error::missing_field("time"))?;
        NaiveDateTime::parse_from_str(format!("{} {}", date, time).as_str(), "%Y-%m-%d %H:%M")
            .map(|date| date.and_local_timezone(Europe__London).single())
            .ok().flatten()
            .map(|tz| tz.with_timezone(&Utc))
            .ok_or_else(|| de::Error::custom("Date/time invalid"))
    }
}

pub fn deserialize_first_arrival<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    deserializer.deserialize_map(FirstVisitor)
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