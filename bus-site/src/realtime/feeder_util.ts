import {db} from "../db.js";
import {DateTime} from "luxon";
import {
    addTimeNaive,
    format_gtfs_time,
    minIndex,
    type Position,
    sqlToLuxon
} from "../routes/api/service/realtime_util.js";
import {LineUtil, Point} from "../leaflet/geometry/index.js";
import groupBy from "object.groupby";

export type TripCandidate = {trip_id: string, direction: number, route: string, times: string, seqs: string, date: number}
export type TripInfo = {candidate: TripCandidate, diff: number, stopIndex: number, route: string[], departureTimes: string[]}
export type FinalTripCandidate = {vehicle: number, trip: TripInfo}
export type TripCandidateList = {vehicle: number, cands: TripInfo[]}

export const lineSegmentQuery = db.prepare(
    `SELECT DISTINCT code as stop_id, lat as y, long as x FROM stances
             WHERE code IN (
                 SELECT stop_id FROM stop_times
                    INNER JOIN main.trips t on t.trip_id = stop_times.trip_id WHERE t.route_id=?)`
)

const minTripInfo = (c1: TripInfo, c2: TripInfo) => c1.diff < c2.diff
const minTripCand = (c1: FinalTripCandidate, c2: FinalTripCandidate) => c1.trip.diff < c2.trip.diff

function minPredicate<T>(arr: T[], comparator: (i1: T, i2: T) => boolean) {
    let lowest = 0
    for(let i = 0; i < arr.length; i++) {
        if(comparator(arr[i], arr[lowest])) lowest = i
    }
    return arr[lowest]
}

export function getTripCandidates<T>(tripQuery: (date: DateTime, startTime: string, endTime: string, route: string) => T[], routeID: string) {
    let candidates: T[]
    let nowDate = DateTime.now()
    let nowDateMinus1hr = nowDate.minus({hour: 1})
    if(nowDateMinus1hr.hour > DateTime.now().hour) {
        // underflow into previous day
        candidates = [
            ...tripQuery(nowDateMinus1hr, addTimeNaive(format_gtfs_time(nowDate), 25), format_gtfs_time(nowDateMinus1hr), routeID),
            ...tripQuery(nowDate, addTimeNaive(format_gtfs_time(nowDate), 1), "00:00:00", routeID)
        ]
    } else {
        candidates = tripQuery(nowDate, addTimeNaive(format_gtfs_time(nowDate), 1), format_gtfs_time(nowDateMinus1hr), routeID)
    }
    return candidates
}

export function getPoints(routeID: string): Record<string, Point> {
    let latLongs = groupBy(
        lineSegmentQuery.all(routeID) as ({stop_id: string} & Position)[],
        ls => ls.stop_id)
    return Object.fromEntries(Object.entries(latLongs).map(([code, details]) => [code, new Point(details[0].x, details[0].y)]))
}

export function getTripInfo(candidate: TripCandidate, points: Record<string, Point>, loc: Point, nowDate: DateTime): TripInfo {
    // out of all line segments for this candidate, find the closest one
    let route = candidate.route.split(",")
    let departureTimes = candidate.times.split(",")
    let segments = [...Array(route.length - 1).keys()].map(i => {
        // default to a very large distance away if something missing
        if(points[route[i]] === undefined || points[route[i+1]] === undefined) return new Point(0, 0)
        return LineUtil.closestPointOnSegment(loc, points[route[i]], points[route[i+1]])
    })
    let segmentDistances = segments.map(segment => loc.distanceTo(segment))
    let index = minIndex(segmentDistances)

    // figure out where the vehicle *would* be right now (min/max at start/end)
    let pct = points[route[index]].distanceTo(segments[index]) / points[route[index]].distanceTo(points[route[index+1]])
    let fromTime = sqlToLuxon(departureTimes[index])
    let toTime = sqlToLuxon(departureTimes[index + 1])
    let current = fromTime.plus({milliseconds: toTime.diff(fromTime).toMillis() * pct})
    let diff = Math.abs(nowDate.diff(current).toMillis())

    // get absolute time in seconds difference
    return {candidate: candidate, diff, stopIndex: index + 1, route, departureTimes}
}

export function assignVehicles(closeness: TripCandidateList[]) {
    closeness = closeness.filter(v => v.cands.length > 0)
    let assignments: Map<number, TripInfo> = new Map<number, TripInfo>()
    while(closeness.length > 0) {
        // Select lowest closeness value, remove values for the specific vehicle, repeat until all values removed
        let perVehicle: FinalTripCandidate[] = closeness.map((closenesses) => {
            return {
                vehicle: closenesses.vehicle,
                trip: minPredicate(closenesses.cands, minTripInfo)
            }
        })
        let lowest = minPredicate(perVehicle, minTripCand)
        assignments.set(lowest.vehicle, lowest.trip)
        // Remove vehicle from closeness
        closeness.splice(closeness.findIndex(v => v.vehicle == lowest.vehicle), 1)
        // Remove assigned trip from each candidate
        closeness.forEach(v => {
            v.cands = v.cands.filter(c => c.candidate.trip_id !== lowest.trip.candidate.trip_id)
        })
        // Remove any vehicle with no candidates
        closeness = closeness.filter(v => v.cands.length > 0)
    }
    return assignments
}