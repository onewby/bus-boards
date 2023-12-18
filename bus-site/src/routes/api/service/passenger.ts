import type {FeedEntity} from "./gtfs-realtime";
import {
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel, VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "./gtfs-realtime";
import {distanceBetween, format_gtfs_time} from "./realtime_util";
import {db} from "../../../db";
import {DateTime} from "luxon";
import type {StopVisits, Vehicles} from "../../../api.type";
import sourceFile from "./passenger-sources.json";

const opsBySource: Record<string, string[]> = sourceFile.sources
const operatorCodes = sourceFile.operators

// Get all second-last stops to get timings from
const stopsQuery = db.prepare(
    `SELECT DISTINCT stop_id FROM (SELECT stop_id, trip_id AS tid FROM stop_times
            WHERE trip_id IN (SELECT trip_id FROM trips WHERE route_id IN (SELECT route_id FROM routes WHERE agency_id=?))
              AND stop_sequence=(SELECT stop_sequence FROM stop_times WHERE trip_id=tid ORDER BY stop_sequence DESC LIMIT 1 OFFSET 1)
             ORDER BY tid, stop_sequence)`
).pluck()

const stopSources: Record<string, string[]> = Object.fromEntries(
    Object.entries(opsBySource).map(([baseURL, sources]) => {
        return [baseURL, sources.flatMap((source) => {
            return stopsQuery.all(source) as string[]
        })]
    })
)

const routesQuery = db.prepare("SELECT route_id, route_short_name FROM routes WHERE agency_id=?")
// TODO Check trip is running on this day
const tripIDQuery = db.prepare("SELECT stop_times.trip_id FROM stop_times INNER JOIN main.trips t on t.trip_id = stop_times.trip_id WHERE stop_id=? AND substr(departure_time, 1, 5)=substr(?, 1, 5) AND route_id=?").pluck()
const tripDetailsQuery = db.prepare("SELECT departure_time FROM stop_times WHERE trip_id=:tid AND (stop_sequence=(SELECT min(stop_sequence) FROM stop_times WHERE trip_id=:tid) OR stop_sequence=(SELECT max(stop_sequence) FROM stop_times WHERE trip_id=:tid)) ORDER BY departure_time").pluck()
const currentStopQuery = db.prepare("SELECT stop_sequence, stop_id, lat, long FROM stop_times INNER JOIN main.stances s on s.code = stop_times.stop_id WHERE trip_id=? AND departure_time <= ? ORDER BY stop_sequence DESC LIMIT 1")

const operatorRoutes: Record<string, Record<string, string>> = Object.fromEntries(Object.entries(operatorCodes).map(([gtfs, [api, _]]) => {
    return [api, Object.fromEntries(routesQuery.all(gtfs).map((obj: any) => [obj.route_short_name, obj.route_id]))]
}))
const operatorCodeToName: Record<string, string> = Object.fromEntries(Object.values(operatorCodes))

export async function load_passenger_sources(): Promise<FeedEntity[]>  {
    return (await Promise.all(Object.entries(stopSources).map(([baseURL, stops]) => get_passenger_source(baseURL, stops)))).flat()
}

async function get_passenger_source(baseURL: string, stops: string[]): Promise<FeedEntity[]> {
    let vehiclesResp = await fetch(`${baseURL}/network/vehicles`)
    if(!vehiclesResp.ok) return []
    let vehicles: Vehicles = await vehiclesResp.json()
    let allEntities = await Promise.all(stops.map(stop => get_passenger_stop(baseURL, stop, vehicles)))
    return allEntities.flat()
}

async function get_passenger_stop(baseURL: string, stop: string, vehicles: Vehicles): Promise<FeedEntity[]> {
    let stopInfoResp = await fetch(`${baseURL}/network/stops/${stop}/visits`)
    if (!stopInfoResp.ok) return [];
    let stopInfo: StopVisits = await stopInfoResp.json()
    if(stopInfo?._embedded?.["timetable:visit"] === undefined) return []
    return stopInfo._embedded["timetable:visit"].filter(v => v.isRealTime).map(visit => {
        let operator = visit._links["transmodel:line"].operator
        let line = visit._links["transmodel:line"].name
        let std = DateTime.fromISO(visit.aimedDepartureTime)
        let etd = DateTime.fromISO(visit.expectedDepartureTime!)

        // also checks the operator is in our list of expected operators
        let routeID = operatorRoutes[operator]?.[line.toUpperCase()]
        if (!routeID) {
            console.log(`could not find route ID: ${operator}, ${line}`)
            return undefined
        }
        let tripID: string | undefined = tripIDQuery.get(stop, format_gtfs_time(std), routeID) as string | undefined
        if (!tripID) {
            console.log(`could not find trip: ${stop}, ${format_gtfs_time(std)}, ${routeID}`)
            return undefined
        }

        let [tripDep, tripArr] = tripDetailsQuery.all({tid: tripID}) as string[]
        let startDate = DateTime.now()
        if (Number(tripArr.split(":")[0]) >= 24) {
            startDate.minus({day: 1})
        }

        // TODO use expectedDepartureTime to get nearest vehicle if possible?
        let currentTimepoint = std.minus(etd.diff(std))

        let gtfsTime = format_gtfs_time(currentTimepoint)
        if (startDate.get("day") !== currentTimepoint.get("day")) {
            gtfsTime = addTimeNaive(gtfsTime, 24)
        }

        // get stop with max time before currentTimepoint (account for night bus?)
        let currentStop = currentStopQuery.get(tripID, gtfsTime) as
            { stop_sequence: number, stop_id: string, lat: number, long: number }
        if (currentStop === undefined) return undefined

        // Find nearest bus in correct direction (outbound/inbound)
        const lineVehicles = vehicles.features.filter(v =>
            v.properties.line === line
            && v.properties.operator === operatorCodeToName[operator]
            && v.properties.direction === visit.direction
        )
        let nearestVehicleIndex = lineVehicles.reduce((prev: [number, number], curr, index) =>
            minByKey(prev, [distanceBetween({
                x: currentStop.long,
                y: currentStop.lat
            }, {
                x: curr?.geometry?.coordinates[0] ?? 0,
                y: curr?.geometry?.coordinates[1] ?? 0
            }), index]), [99999999, -1])
        let nearestVehicle = lineVehicles[nearestVehicleIndex[1]]

        const updateTime = Date.now() / 1000

        return {
            id: visit._links["timetable:journey"].id,
            alert: undefined,
            isDeleted: false,
            tripUpdate: undefined,
            vehicle: {
                trip: {
                    tripId: tripID,
                    routeId: routeID,
                    directionId: -1,
                    startTime: tripDep,
                    startDate: startDate.toISODate({format: 'basic'})!,
                    scheduleRelationship: visit.cancelled ? TripDescriptor_ScheduleRelationship.CANCELED : TripDescriptor_ScheduleRelationship.SCHEDULED
                },
                vehicle: undefined,
                position: nearestVehicle ? {
                    latitude: nearestVehicle.geometry.coordinates[1],
                    longitude: nearestVehicle.geometry.coordinates[0],
                    bearing: nearestVehicle.properties.bearing ?? 0,
                    odometer: -1,
                    speed: -1
                } : undefined,
                currentStopSequence: currentStop?.stop_sequence,
                stopId: currentStop?.stop_id,
                currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                timestamp: updateTime,
                congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
            }
        }
    }).filter(data => data !== undefined) as FeedEntity[]
}

const addTimeNaive = (time: string, add: number) => (Number(time.substring(0, 2)) + add).toString().padStart(2, "0") + time.substring(2, time.length)
function minByKey<T>(v1: [number, T], v2: [number, T]) {
    return v1[0] < v2[0] ? v1 : v2
}