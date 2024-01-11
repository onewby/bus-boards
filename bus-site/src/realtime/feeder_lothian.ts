import {type DownloadResponse, UpdateFeeder} from "./feeder.js";
import {db} from "../db.js";
import type {LothianEvent, LothianEvents, LothianLiveVehicles} from "../api.type.ts";
import {DateTime} from "luxon";
import {
    assignVehicles,
    getPoints,
    getTripCandidates,
    getTripInfo,
    type TripCandidate,
    type TripCandidateList
} from "./feeder_util.js";
import {Point} from "../leaflet/geometry/index.js";
import {
    type Alert,
    Alert_Cause,
    Alert_Effect,
    type FeedEntity,
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel,
    VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "../routes/api/service/gtfs-realtime.js";
import groupBy from "object.groupby";
import {download_route_data} from "../../import_lothian.ts";

const getAllPatterns = () => db.prepare("SELECT * FROM lothian").all() as {pattern: string, route: string}[]

const currentTripsQuery = (date: DateTime, startTime: string, endTime: string, pattern: string) => db.prepare(
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
                   AND ((start_date <= :date AND end_date >= :date AND ${date.weekdayLong!.toLowerCase()}=1) OR exception_type=1)
                   AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                   AND start.departure_time <= :startTime AND finish.departure_time >= :endTime`
).all({date: Number(date.toFormat("yyyyMMdd")), startTime, endTime, pattern}) as TripCandidate[]

// Grouped to reduce parallelism a bit to lessen chance of rate limiting
const patterns = groupBy(getAllPatterns(), p => p.route)

export async function load_Lothian_vehicles(): Promise<DownloadResponse> {
    let gtfsRT: FeedEntity[] = []

    const nowDate = DateTime.now()
    const gtfsAlertsByRoute = await load_lothian_alerts()

    await Promise.allSettled(Object.values(patterns).map(async route => {
        for(const pattern of route) {
            const routeName = pattern.pattern.split(':')[0]
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
                    alert: gtfsAlertsByRoute[routeName],
                    isDeleted: false,
                    tripUpdate: undefined,
                    vehicle: {
                        trip: {
                            tripId: trip.candidate.trip_id,
                            routeId: pattern.route,
                            directionId: -1,
                            startTime: trip.departureTimes[trip.stopIndex],
                            startDate: DateTime.fromFormat(String(trip.candidate.date), "yyyyMMdd").toISODate()!,
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
    })).then(results => {
        results.forEach(result => {
            if(result.status === "rejected") console.error(result.reason)
        })
    })

    return {
        entities: gtfsRT,
        stopAlerts: {}
    }
}

async function load_lothian_alerts(): Promise<Record<string, Alert>> {
    const serviceAlerts: LothianEvents = await (await fetch("https://lothianupdates.com/api/public/getServiceUpdates")).json()
    const lothianAlertsByRoute: Record<string, LothianEvent[]> = {}
    serviceAlerts.events.forEach((event) => {
        event.routes_affected.forEach(route => {
            if (!lothianAlertsByRoute[route.name]) lothianAlertsByRoute[route.name] = []
            lothianAlertsByRoute[route.name].push(event)
        });
    })
    return Object.fromEntries(Object.entries(lothianAlertsByRoute).map(([route, events]) => {
        events = events.filter(event => event.time_ranges.findIndex((time) => {
            let start = time.start ? DateTime.fromISO(time.start) : DateTime.fromSeconds(0)
            let end = time.finish ? DateTime.fromISO(time.finish) : DateTime.now().plus({year: 1})
            return DateTime.now() >= start && DateTime.now() <= end
        }) > -1)
        if(events.length === 0) return []
        // GTFS alert semantics overridden to encode multiple alerts compactly
        return [route, {
            activePeriod: {
                start: 0,
                end: DateTime.now().plus({year: 1}).toSeconds()
            },
            cause: Alert_Cause.OTHER_CAUSE,
            descriptionText: {translation: events.map(event => ({text: event.description.en, language: "en"}))},
            effect: Alert_Effect.OTHER_EFFECT,
            headerText: {translation: events.map(event => ({text: event.title.en, language: "en"}))},
            informedEntity: [],
            url: {translation: events.map(event => ({text: event.url ?? "", language: "en"}))}
        }]
    }).filter(entry => entry.length !== 0))
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