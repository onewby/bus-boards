use std::cmp::Ordering;
use std::collections::HashMap;
use config::Map;
use geo_types::Point;
use chrono::{DateTime, TimeDelta, Timelike, Utc};
use std::sync::Arc;
use std::ops::{Add, Sub};
use geo::{EuclideanDistance, LineLocatePoint};
use itertools::Itertools;
use crate::db::DBPool;
use crate::util;
use crate::util::f64_cmp;

type TripCandidateDBFunction = fn(&Arc<DBPool>, &DateTime<Utc>, i64, i64, &str) -> Vec<TripCandidate>;

pub fn get_trip_candidates(db: &Arc<DBPool>, specifier: &str, now_date: &DateTime<Utc>, trip_query: TripCandidateDBFunction) -> Vec<TripCandidate> {
    let now_date_secs = util::zero_day(now_date).timestamp();
    let now_date_minus_1hr = now_date.sub(TimeDelta::hours(1));
    let now_date_minus_1hr_secs = util::zero_day(&now_date_minus_1hr).timestamp();
    let now_date_plus_1hr = now_date.add(TimeDelta::hours(1));
    let now_date_plus_1hr_secs = util::zero_day(&now_date_plus_1hr).timestamp();

    if now_date_minus_1hr.hour() > now_date.hour() {
        let mut cands = trip_query(db, &now_date_minus_1hr, now_date_minus_1hr_secs + 7200, now_date_minus_1hr_secs, specifier);
        cands.extend(trip_query(db, now_date, now_date_secs + 3600, 0, specifier));
        cands
    } else if now_date_plus_1hr.hour() < now_date.hour() {
        let mut cands = trip_query(db, now_date, now_date_secs + 3600, now_date_secs - 3600, specifier);
        cands.extend(trip_query(db, now_date, now_date_plus_1hr_secs, now_date_plus_1hr_secs - 7200, specifier));
        cands
    } else {
        trip_query(db, now_date, now_date_plus_1hr_secs, now_date_minus_1hr_secs, specifier)
    }
}

pub fn get_trip_info<'a>(candidate: &'a TripCandidate, candidate_i: usize, points: &Map<String, Point>, loc: &Point, now: &DateTime<Utc>) -> TripInfo {
    let route = &candidate.route;
    let segments: Vec<geo_types::Line<f64>> = (0..route.len()-1).map(|i| {
        geo_types::Line::new(points.get(&route[i]).map(|p| p.clone()).unwrap_or_default(), points.get(&route[i+1]).map(|p| p.clone()).unwrap_or_default())
    }).collect();
    let closest_segment = segments.iter().map(|s| loc.euclidean_distance(s)).position_min_by(f64_cmp).unwrap_or(0);

    let pct = segments[closest_segment].line_locate_point(loc).unwrap_or(0.0);

    let from_time = candidate.times[closest_segment];
    let to_time = candidate.times[closest_segment + 1];

    let current_time = from_time.add(TimeDelta::milliseconds((to_time.signed_duration_since(from_time).num_milliseconds() as f64 * pct) as i64));
    let diff = now.signed_duration_since(current_time).num_milliseconds().abs() as usize;

    return TripInfo {
        candidate: candidate_i,
        diff,
        stop_index: closest_segment + 1
    }
}

pub fn assign_vehicles(mut closeness: &mut Vec<TripCandidateList>, candidates: &Vec<TripCandidate>) -> HashMap<usize, TripInfo> {
    let mut assignments: HashMap<usize, TripInfo> = HashMap::new();
    // Until empty
    while closeness.len() > 0 {
        // Find closest candidate for each vehicle
        let per_vehicle = closeness.iter().map(|c| FinalTripCandidate {
            vehicle: c.vehicle,
            // Find closest matching remaining trip
            trip: c.cands.iter().min().unwrap().clone(),
        });

        // Find closest match of these
        let lowest = per_vehicle.min().unwrap();
        // This is our assignment
        assignments.insert(lowest.vehicle, lowest.trip);

        // Now it's assigned, remove the vehicle from contention
        closeness.remove(closeness.iter().position(|v| v.vehicle == lowest.vehicle).unwrap());
        // Remove the trip from any other candidates
        closeness.iter_mut().for_each(|v| {
            v.cands.retain(|c| candidates[c.candidate].trip_id != candidates[lowest.trip.candidate].trip_id)
        });
        // And remove any vehicles without any further hope
        closeness.retain(|v| v.cands.len() > 0);
    }
    assignments
}

#[derive(Debug)]
pub struct TripCandidate {
    pub trip_id: String,
    pub direction: Option<u8>,
    pub route: Vec<String>,
    pub times: Vec<DateTime<Utc>>,
    pub seqs: Vec<u32>,
    pub date: usize
}

#[derive(Copy, Clone)]
pub struct TripInfo {
    pub(crate) candidate: usize,
    diff: usize,
    pub(crate) stop_index: usize
}

impl Eq for TripInfo {}

impl PartialEq<Self> for TripInfo {
    fn eq(&self, other: &Self) -> bool {
        self.diff.eq(&other.diff)
    }
}

impl PartialOrd<Self> for TripInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.diff.partial_cmp(&other.diff)
    }
}

impl Ord for TripInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.diff.cmp(&other.diff)
    }
}

pub struct FinalTripCandidate {
    vehicle: usize,
    trip: TripInfo
}

impl Eq for FinalTripCandidate {}

impl PartialEq<Self> for FinalTripCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.trip.eq(&other.trip)
    }
}

impl PartialOrd<Self> for FinalTripCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.trip.partial_cmp(&other.trip)
    }
}

impl Ord for FinalTripCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.trip.cmp(&other.trip)
    }
}

pub struct TripCandidateList {
    pub vehicle: usize,
    pub cands: Vec<TripInfo>
}
