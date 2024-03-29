import {type DownloadResponse, UpdateFeeder} from "./feeder";
import {db} from "../db";
import type {LothianEvents, LothianLiveVehicles} from "../api.type";
import {DateTime} from "luxon";
import {
    assignVehicles,
    getPoints,
    getTripCandidates,
    getTripInfo, mapSQLTripCandidate,
    type TripCandidate,
    type TripCandidateList
} from "./feeder_util";
import {Point} from "../leaflet/geometry/index.js";
import {
    type Alert,
    alert_CauseFromJSON,
    alert_EffectFromJSON,
    type FeedEntity,
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel,
    VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "../routes/api/service/gtfs-realtime";
import groupBy from "object.groupby";
import {download_route_data, lothianOpCodes} from "../../import_lothian";
import {format_gtfs_time, ZERO_TIME} from "../routes/api/service/realtime_util.ts";

const getAllPatterns = () => db.prepare("SELECT * FROM lothian").all() as {pattern: string, route: string}[]

const currentTripsQueryStmt = db.prepare(
    `SELECT trips.trip_id, :date as date,
                (SELECT group_concat(stop_id) FROM (SELECT stop_id FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as route,
                (SELECT group_concat(departure_time) FROM (SELECT departure_time FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as times,
                (SELECT group_concat(stop_sequence) FROM (SELECT stop_sequence FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as seqs
                 FROM polar
                   INNER JOIN main.trips trips on polar.gtfs = trips.trip_id
                   LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                   LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
                   INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                   INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
                 WHERE direction IS NULL AND polar=:pattern
                   AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                   AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                   AND +start.departure_time <= :startTime AND +finish.departure_time >= :endTime`
)
const currentTripsQuery = (date: DateTime, startTime: number, endTime: number, pattern: string) => {
    let dateSecs = date.set(ZERO_TIME).toSeconds()
    return currentTripsQueryStmt.all({
        date: Number(date.toFormat("yyyyMMdd")),
        day: date.weekday - 1,
        startTime,
        endTime,
        pattern
    }).map(obj => mapSQLTripCandidate(obj, dateSecs));
}

const getRouteInfoStmt = db.prepare("SELECT route_id, agency_id FROM routes WHERE route_short_name=? AND agency_id IN (?, ?, ?)")
const getRouteInfo = (route: string) => getRouteInfoStmt.get(route, ...lothianOpCodes) as {route_id: string, agency_id: string}

// Grouped to reduce parallelism a bit to lessen chance of rate limiting
const patterns = groupBy(getAllPatterns(), p => p.route)

export async function load_Lothian_vehicles(): Promise<DownloadResponse> {
    let gtfsRT: FeedEntity[] = []

    const nowDate = DateTime.now()

    await Promise.allSettled(Object.values(patterns).map(async route => {
        for(const pattern of route) {
            let vehicles: LothianLiveVehicles
            try {
                vehicles = await (await fetchWithTimeout(`https://tfeapp.com/api/website/vehicles_on_route.php?route_id=${pattern.pattern}`, {timeout: 10000})).json() as LothianLiveVehicles
            } catch (e) {
                return
            }
            const candidates = getTripCandidates(currentTripsQuery, pattern.pattern)
            const points = getPoints(pattern.route)

            // Find closeness to each candidate
            let closeness: TripCandidateList[] = vehicles.vehicles.map((vehicle, i) => {
                let loc = new Point(vehicle.longitude, vehicle.latitude)
                return {
                    vehicle: i,
                    cands: candidates.map(candidate => getTripInfo(candidate, points, loc, nowDate))
                }
            })

            // Assign vehicles to trips via closeness (closest assigned first)
            let assignments = assignVehicles(closeness)

            // Generate GTFS from each
            gtfsRT.push(...[...assignments.entries()].map(([vehicleIndex, trip]) => {
                const updateTime = Date.now() / 1000

                return {
                    id: vehicles.vehicles[vehicleIndex].vehicle_id,
                    alert: undefined,
                    isDeleted: false,
                    tripUpdate: undefined,
                    vehicle: {
                        trip: {
                            tripId: trip.candidate.trip_id,
                            routeId: pattern.route,
                            directionId: -1,
                            startTime: format_gtfs_time(trip.departureTimes[0]),
                            startDate: String(trip.candidate.date)!,
                            scheduleRelationship: TripDescriptor_ScheduleRelationship.SCHEDULED
                        },
                        vehicle: undefined,
                        position: {
                            latitude: vehicles.vehicles[vehicleIndex].latitude,
                            longitude: vehicles.vehicles[vehicleIndex].longitude,
                            bearing: vehicles.vehicles[vehicleIndex].heading,
                            odometer: -1,
                            speed: -1
                        },
                        currentStopSequence: Number(trip.candidate.seqs[trip.stopIndex]),
                        stopId: trip.route[trip.stopIndex],
                        currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                        timestamp: updateTime,
                        congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                        occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
                    }
                }
            }))
        }
    })).then(results => {
        results.forEach(result => {
            if(result.status === "rejected") console.error(result.reason)
        })
    })

    return {
        entities: gtfsRT,
        alerts: await load_lothian_alerts()
    }
}

async function load_lothian_alerts(): Promise<Alert[]> {
    const serviceAlerts: LothianEvents = await (await fetch("https://lothianupdates.com/api/public/getServiceUpdates")).json()
    return serviceAlerts.events.map(event => ({
        activePeriod: event.time_ranges.map(active => ({
            start: active.start ? DateTime.fromISO(active.start).toSeconds() : 0,
            end: active.finish ? DateTime.fromISO(active.finish).toSeconds() : DateTime.now().plus({year: 1}).toSeconds()
        })),
        cause: alert_CauseFromJSON(event.cause),
        effect: alert_EffectFromJSON(event.effect),
        descriptionText: {translation: [{text: event.description.en, language: "en"}]},
        headerText: {translation: [{text: event.title.en, language: "en"}]},
        informedEntity: event.routes_affected.map(entity => {
            let route = getRouteInfo(entity.name)
            return { routeId: route.route_id };
        }),
        url: {translation: [{text: event.url ?? "", language: "en"}]}
    }))
}

// https://dmitripavlutin.com/timeout-fetch-request/
async function fetchWithTimeout(resource: string, options: {timeout?: number} = {}) {
    const { timeout = 8000 } = options;

    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), timeout);

    const response = await fetch(resource, {
        ...options,
        signal: controller.signal
    });
    clearTimeout(id);

    return response;
}

new UpdateFeeder(load_Lothian_vehicles, download_route_data).init()