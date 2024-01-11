import {downloadRouteDirections} from "../../import_passenger.js";
import {readFileSync, writeFileSync} from "fs";
import {existsSync} from "node:fs";
import {
    FeedEntity,
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel, VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "../routes/api/service/gtfs-realtime.js";
import {db} from "../db.js";
import {DateTime} from "luxon";
import type {Vehicles} from "../api.type";
import sourceFile from "../routes/api/service/passenger-sources.json" assert {type: 'json'};
import groupBy from "object.groupby";
import {Point} from "../leaflet/geometry/index.js"
import {type DownloadResponse, Feeder} from "./feeder.js";
import {
    assignVehicles,
    getPoints,
    getTripCandidates, getTripInfo,
    type TripCandidate,
    type TripCandidateList
} from "./feeder_util.js";

const routeIDQuery = db.prepare("SELECT route_id FROM routes WHERE agency_id=? AND route_short_name=?").pluck()

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

export async function load_passenger_sources(): Promise<DownloadResponse>  {
    let entities = (await Promise.all(Object.keys(sourceFile.sources).map(baseURL => get_passenger_source(baseURL as (keyof typeof sourceFile.sources))))).flat()
    return { entities, stopAlerts: {} }
}

export async function get_passenger_source(baseURL: keyof typeof sourceFile.sources) {
    let vehiclesResp = await fetch(`${baseURL}/network/vehicles`)
    if(!vehiclesResp.ok) return []
    return process_vehicles(await vehiclesResp.json() as Vehicles, sourceFile.sources[baseURL] as (keyof typeof sourceFile.operators)[])
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
            let nowDate = DateTime.now()
            let candidates: TripCandidate[] = getTripCandidates(currentTripsQuery, routeID)

            // Find closeness to each trip

            let points = getPoints(routeID)
            // Calculate all closeness values (create vehicle-candidate table)
            let closeness: TripCandidateList[] = lineVehicles.map((vehicle, i) => {
                let loc = new Point(vehicle.geometry.coordinates[0], vehicle.geometry.coordinates[1])
                let direction = vehicle.properties.direction === 'inbound' ? 0 : 1
                return {vehicle: i, cands: candidates.filter(c => c.direction === direction)
                        .map(candidate => getTripInfo(candidate, points, loc, nowDate))};
            })

            // Assign vehicles to trips via closeness (closest assigned first)
            let assignments = assignVehicles(closeness)

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


class PassengerFeeder extends Feeder {

    lastUpdate = existsSync(".update") ? DateTime.fromISO(readFileSync(".update", "utf-8")) : DateTime.now().minus({days: 5, hours: 1})

    async checkPassengerUpdate() {
        if (this.lastUpdate.diffNow("days").days <= -5) {
            await downloadRouteDirections()
            this.lastUpdate = DateTime.now().set({hour: 2, minute: 0, second: 0, millisecond: 0})
            writeFileSync(".update", DateTime.now().toISO()!)
        }
    }

    constructor() {
        super(async () => {
            await this.checkPassengerUpdate()
            return load_passenger_sources()
        });
    }
}

new PassengerFeeder().init()