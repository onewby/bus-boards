use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Add, Sub};
use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Timelike, Utc};
use config::Map;
use geo::{EuclideanDistance, LineLocatePoint};
use geo_types::Point;
use itertools::Itertools;

use util::zero_day;

use crate::db::DBPool;
use crate::util;
use crate::util::f64_cmp;

type TripCandidateDBFunction = fn(&Arc<DBPool>, &DateTime<Utc>, i64, i64, &str) -> Vec<TripCandidate>;

/// Get a list of GTFS trips that could match to a realtime vehicle (within a time window +-1hr)
pub fn get_trip_candidates(db: &Arc<DBPool>, specifier: &str, now_date: &DateTime<Utc>, trip_query: TripCandidateDBFunction) -> Vec<TripCandidate> {
    // Get times zero'd to match the date-generic GTFS database
    let now_date_secs = zero_day(now_date).timestamp();
    let now_date_minus_1hr = now_date.sub(TimeDelta::hours(1));
    let now_date_minus_1hr_secs = zero_day(&now_date_minus_1hr).timestamp();
    let now_date_plus_1hr = now_date.add(TimeDelta::hours(1));
    let now_date_plus_1hr_secs = zero_day(&now_date_plus_1hr).timestamp();

    if now_date_minus_1hr.hour() > now_date.hour() {
        // Time window underflows into the previous day - also consider departure times from the previous day over 24hrs
        let mut cands = trip_query(db, &now_date_minus_1hr, now_date_minus_1hr_secs + 7200, now_date_minus_1hr_secs, specifier);
        cands.extend(trip_query(db, now_date, now_date_secs + 3600, 0, specifier));
        cands
    } else if now_date_plus_1hr.hour() < now_date.hour() {
        // Time window overflows into the subsequent day - also consider early departure times in the next day
        let mut cands = trip_query(db, now_date, now_date_secs + 3600, now_date_secs - 3600, specifier);
        cands.extend(trip_query(db, now_date, now_date_plus_1hr_secs, now_date_plus_1hr_secs - 7200, specifier));
        cands
    } else {
        // Time window is fully within a single day
        trip_query(db, now_date, now_date_plus_1hr_secs, now_date_minus_1hr_secs, specifier)
    }
}

/// For a given vehicle-trip combination, get its estimated next stop and how delayed the vehicle would be right now if running the route
pub fn get_trip_info(candidate: &TripCandidate, candidate_i: usize, points: &Map<String, Point>, loc: &Point, now: &DateTime<Utc>) -> TripInfo {
    // Find the closest route line segment to the vehicle
    let route = &candidate.route;
    let segments: Vec<geo_types::Line<f64>> = (0..route.len()-1).map(|i| {
        geo_types::Line::new(points.get(&route[i]).copied().unwrap_or_default(), points.get(&route[i+1]).copied().unwrap_or_default())
    }).collect();
    let closest_segment = segments.iter().map(|s| loc.euclidean_distance(s)).position_min_by(f64_cmp).unwrap_or(0);

    // Find how far along the line segment this point is
    let pct = segments[closest_segment].line_locate_point(loc).unwrap_or(0.0);

    // Get departure times from the two ends of the line segment
    let from_time = candidate.times[closest_segment];
    let to_time = candidate.times[closest_segment + 1];

    // Interpolate to find the time the vehicle would be expected to be in that position
    let current_time = from_time.add(TimeDelta::milliseconds((to_time.signed_duration_since(from_time).num_milliseconds() as f64 * pct) as i64));
    // Find the difference between the expected time for the vehicle's position and the actual time
    let diff = now.signed_duration_since(current_time).num_milliseconds().unsigned_abs() as usize;

    TripInfo {
        candidate: candidate_i,
        diff,
        stop_index: closest_segment + 1
    }
}

/// Assign vehicles to GTFS trip candidates
pub fn assign_vehicles(closeness: &mut Vec<TripCandidateList>, candidates: &[TripCandidate]) -> HashMap<usize, TripInfo> {
    let mut assignments: HashMap<usize, TripInfo> = HashMap::new();
    // Until empty
    while !closeness.is_empty() {
        // Find closest candidate for each vehicle
        let per_vehicle = closeness.iter().map(|c| FinalTripCandidate {
            vehicle: c.vehicle,
            // Find closest matching remaining trip
            trip: *c.cands.iter().min().unwrap(),
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
        closeness.retain(|v| !v.cands.is_empty());
    }
    assignments
}

/// Potential GTFS trip candidate
#[derive(Debug)]
pub struct TripCandidate {
    pub trip_id: String,
    pub direction: Option<u8>,
    pub route: Vec<String>,
    pub times: Vec<DateTime<Utc>>,
    pub seqs: Vec<u32>,
    pub date: usize
}

/// Vehicle delay and expected next stop for a given vehicle-trip combination
#[derive(Copy, Clone)]
pub struct TripInfo {
    pub candidate: usize,
    pub diff: usize,
    pub stop_index: usize
}

// For vehicle assignment .min() - order by diff

impl Eq for TripInfo {}

impl PartialEq<Self> for TripInfo {
    fn eq(&self, other: &Self) -> bool {
        self.diff.eq(&other.diff)
    }
}

impl PartialOrd<Self> for TripInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TripInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.diff.cmp(&other.diff)
    }
}

/// Vehicle index and its assigned GTFS trip
pub struct FinalTripCandidate {
    vehicle: usize,
    trip: TripInfo
}

// For vehicle assignment .min() - order by trip diff

impl Eq for FinalTripCandidate {}

impl PartialEq<Self> for FinalTripCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.trip.eq(&other.trip)
    }
}

impl PartialOrd<Self> for FinalTripCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FinalTripCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.trip.cmp(&other.trip)
    }
}

/// Vehicle realtime index and a list of candidate GTFS trips
pub struct TripCandidateList {
    pub vehicle: usize,
    pub cands: Vec<TripInfo>
}
