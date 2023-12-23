import type {FeedEntity} from "./gtfs-realtime";
import {
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel, VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "./gtfs-realtime";
import {distanceBetween, findNearestSegmentPoint, format_gtfs_time, type Position} from "./realtime_util";
import {db} from "../../../db";
import {DateTime} from "luxon";
import type {Vehicles} from "../../../api.type";
import sourceFile from "./passenger-sources.json";
import groupBy from "object.groupby";

const routeIDQuery = db.prepare("SELECT route_id FROM routes WHERE agency_id=? AND route_short_name=?").pluck()
const lineSegmentQuery = db.prepare(
    `SELECT DISTINCT code as stop_id, lat as y, long as x FROM stances
             WHERE code IN (
                 SELECT stop_id FROM stop_times
                    INNER JOIN main.trips t on t.trip_id = stop_times.trip_id WHERE t.route_id=?)`
)
const currentTripsQuery = (date: DateTime, startTime: string, endTime: string, route: string) => db.prepare(
    `SELECT trips.trip_id, p.direction, :date as date,
                (SELECT group_concat(stop_id) FROM (SELECT stop_id FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as route,
                (SELECT group_concat(departure_time) FROM (SELECT departure_time FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as times,
                (SELECT group_concat(stop_sequence) FROM (SELECT stop_sequence FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as seqs
                 FROM trips
                          INNER JOIN main.routes r on r.route_id = trips.route_id
                          LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                          LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
                          LEFT OUTER JOIN main.polar p on trips.trip_id = p.gtfs
                          INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                          INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
                 WHERE r.route_id=:route
                   AND ((start_date <= :date AND end_date >= :date AND ${date.weekdayLong!.toLowerCase()}=1) OR exception_type=1)
                   AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                   AND start.departure_time <= :startTime AND finish.departure_time >= :endTime`
).all({date: Number(date.toFormat("yyyyMMdd")), startTime, endTime, route}) as TripCandidate[]

type TripCandidate = {trip_id: string, direction: number, route: string, times: string, seqs: string, date: number}
type TripInfo = {candidate: TripCandidate, diff: number, stopIndex: number, route: string[], departureTimes: string[]}
type FinalTripCandidate = {vehicle: number, trip: TripInfo}
type TripCandidateList = {vehicle: number, cands: TripInfo[]}

const minTripInfo = (c1: TripInfo, c2: TripInfo) => c1.diff < c2.diff
const minTripCand = (c1: FinalTripCandidate, c2: FinalTripCandidate) => c1.trip.diff < c2.trip.diff

export async function load_passenger_sources(): Promise<FeedEntity[]>  {
    return (await Promise.all(Object.keys(sourceFile.sources).map(baseURL => get_passenger_source(baseURL as (keyof typeof sourceFile.sources))))).flat()
}

export async function get_passenger_source(baseURL: keyof typeof sourceFile.sources) {
    let vehiclesResp = await fetch(`${baseURL}/network/vehicles`)
    if(!vehiclesResp.ok) return []
    return process_vehicles(await vehiclesResp.json(), sourceFile.sources[baseURL] as (keyof typeof sourceFile.operators)[])
}

async function process_vehicles(vehicles: Vehicles, operators: (keyof typeof sourceFile.operators)[]): Promise<FeedEntity[]> {
    let gtfsRT: FeedEntity[] = []
    let byOperator = groupBy(vehicles.features, (vehicle) => vehicle.properties.operator)
    for(let operator of operators) {
        let opVehicles = byOperator[operator]
        if(opVehicles === undefined) continue;
        let byLine = groupBy(opVehicles, (vehicle) => vehicle.properties.line)
        for(let line of Object.keys(byLine)) {
            let lineVehicles = byLine[line]
            let routeID = routeIDQuery.get(sourceFile.operators[operator].gtfs, line) as string

            // Get all candidate trips
            let candidates: TripCandidate[] = []
            let nowDate = DateTime.now()
            let nowDateMinus1hr = nowDate.minus({hour: 1})
            if(nowDateMinus1hr.hour > DateTime.now().hour) {
                // underflow into previous day
                candidates = [
                    ...currentTripsQuery(nowDateMinus1hr, addTimeNaive(format_gtfs_time(nowDate), 25), format_gtfs_time(nowDateMinus1hr), routeID),
                    ...currentTripsQuery(nowDate, addTimeNaive(format_gtfs_time(nowDate), 1), "00:00:00", routeID)
                ]
            } else {
                candidates = currentTripsQuery(nowDate, addTimeNaive(format_gtfs_time(nowDate), 1), format_gtfs_time(nowDateMinus1hr), routeID)
            }

            // Find closeness to each trip
            let latLongs = groupBy(
                lineSegmentQuery.all(routeID) as ({stop_id: string} & Position)[],
                ls => ls.stop_id)

            // Calculate all closeness values (create vehicle-candidate table)
            let closeness: TripCandidateList[] = lineVehicles.map((vehicle, i) => {
                let loc = {x: vehicle.geometry.coordinates[0], y: vehicle.geometry.coordinates[1]}
                let direction = vehicle.properties.direction === 'inbound' ? 0 : 1
                return {vehicle: i, cands: candidates.filter(c => c.direction === direction).map(candidate => {
                    // out of all line segments for this candidate, find the closest one
                    let route = candidate.route.split(",")
                    let departureTimes = candidate.times.split(",")
                    let segments = [...Array(route.length - 1).keys()].map(i => {
                        return findNearestSegmentPoint(loc, latLongs[route[i]][0], latLongs[route[i+1]][0])
                    })
                    let segmentDistances = segments.map(segment => {
                        return distanceBetween(loc, segment)
                    })
                    let index = minIndex(segmentDistances)

                    // figure out where the vehicle *would* be right now (min/max at start/end)
                    let pct = distanceBetween(latLongs[route[index]][0], segments[index]) / distanceBetween(latLongs[route[index]][0], latLongs[route[index+1]][0])
                    let fromTime = sqlToLuxon(departureTimes[index])
                    let toTime = sqlToLuxon(departureTimes[index + 1])
                    let current = fromTime.plus({milliseconds: toTime.diff(fromTime).toMillis() * pct})
                    let diff = Math.abs(nowDate.diff(current).toMillis())

                    // get absolute time in seconds difference
                    return {candidate: candidate, diff, stopIndex: index, route, departureTimes}
                })};
            }).filter(v => v.cands.length > 0)

            // Assign vehicles to trips via closeness (closest assigned first)
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

            // Generate GTFS from each
            gtfsRT.push(...[...assignments.entries()].map(([vehicleIndex, trip]) => {
                const updateTime = Date.now() / 1000

                return {
                    id: lineVehicles[vehicleIndex].properties.vehicle,
                    alert: undefined,
                    isDeleted: false,
                    tripUpdate: undefined,
                    vehicle: {
                        trip: {
                            tripId: trip.candidate.trip_id,
                            routeId: routeID,
                            directionId: -1,
                            startTime: trip.departureTimes[trip.stopIndex],
                            startDate: DateTime.fromFormat(String(trip.candidate.date), "yyyyMMdd").toISODate()!,
                            scheduleRelationship: TripDescriptor_ScheduleRelationship.SCHEDULED
                        },
                        vehicle: undefined,
                        position: {
                            latitude: lineVehicles[vehicleIndex].geometry.coordinates[1],
                            longitude: lineVehicles[vehicleIndex].geometry.coordinates[0],
                            bearing: lineVehicles[vehicleIndex].properties.bearing ?? 0,
                            odometer: -1,
                            speed: -1
                        },
                        currentStopSequence: Number(trip.candidate.seqs.split(",")[trip.stopIndex]),
                        stopId: trip.route[trip.stopIndex],
                        currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                        timestamp: updateTime,
                        congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                        occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
                    }
                }
            }))
        }
    }
    return gtfsRT
}

const addTimeNaive = (time: string, add: number) => (Number(time.substring(0, 2)) + add).toString().padStart(2, "0") + time.substring(2, time.length)

function minIndex(arr: any[]) {
    let lowest = 0
    for(let i = 0; i < arr.length; i++) {
        if(arr[i] < arr[lowest]) lowest = i
    }
    return lowest
}

function minPredicate<T>(arr: T[], comparator: (i1: T, i2: T) => boolean) {
    let lowest = 0
    for(let i = 0; i < arr.length; i++) {
        if(comparator(arr[i], arr[lowest])) lowest = i
    }
    return arr[lowest]
}

function sqlToLuxon(time: string) {
    let days = Math.floor(Number(time.substring(0, 2)) / 24)
    let newTime = addTimeNaive(time, -24 * days)
    return DateTime.fromSQL(newTime).plus({days})
}