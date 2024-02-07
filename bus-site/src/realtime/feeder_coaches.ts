import {db} from "../db";
import {
    type FeedEntity,
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel,
    VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "../routes/api/service/gtfs-realtime";
import type {MegabusVehicles} from "../api.type";
import {DateTime} from "luxon";
import {
    dayDiff,
    format_gtfs_time, minIndex,
    type Position, relativeTo, ZERO_DAY
} from "../routes/api/service/realtime_util";
import groupBy from "object.groupby";
import {lineSegmentQuery} from "./feeder_util";
import {Point, LineUtil} from "../leaflet/geometry/index.js"
import type {ChronologicalDeparture} from "../api.type";
import {type DownloadResponse, Feeder} from "./feeder";

const coachOperators = ["OP564", "OP545", "OP563", "OP567"]

type Route = {agency_id: string, route_id: string, route_short_name: string}
let routes = db.prepare(
    `SELECT agency_id, route_id, route_short_name FROM routes WHERE agency_id IN (${coachOperators.map(_ => "?").join(",")})`
).all(...coachOperators) as Route[]

type Trip = {trip_id: string, route: string, times: string, seqs: string}
const findTripQuery = db.prepare(
    `SELECT trips.trip_id,
            (SELECT group_concat(stop_id) FROM (SELECT stop_id FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as route,
            (SELECT group_concat(departure_time) FROM (SELECT departure_time FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as times,
            (SELECT group_concat(stop_sequence) FROM (SELECT stop_sequence FROM stop_times WHERE trip_id=trips.trip_id ORDER BY stop_sequence)) as seqs,
            (SELECT locality_name FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=std.stop_id) as stdLoc,
            (SELECT locality_name FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=sta.stop_id) as staLoc
            FROM trips
                INNER JOIN main.stop_times std on (trips.trip_id = std.trip_id AND std.stop_sequence=min_stop_seq)
                INNER JOIN main.stop_times sta on (trips.trip_id = sta.trip_id AND sta.stop_sequence=max_stop_seq)
                LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
            WHERE route_id=:route AND std.departure_time=:startTime AND sta.departure_time=:endTime
                AND stdLoc LIKE :depWildcard AND staLoc LIKE :arrWildcard
                AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                    AND NOT (exception_type IS NOT NULL AND exception_type = 2)`
)
const findTrip = (date: DateTime, route: string, startTime: number, endTime: number, origin: string, dest: string) => findTripQuery.get({date: Number(date.toFormat("yyyyMMdd")), route, startTime, endTime, depWildcard: origin.split(" (")[0] + '%', arrWildcard: dest.split(" (")[0] + '%', day: date.weekday - 1}) as Trip | undefined

export async function load_coaches(): Promise<DownloadResponse> {
    let config = await (await fetch("https://coachtracker.uk.megabus.com/configs/global.js")).text()
    let apiURL = config.match(/\s*API_URL: '(.*)',/)?.[1] ?? ""
    let apiKey = config.match(/\s*API_KEY: '(.*)',/)?.[1] ?? ""

    const timeFrom = DateTime.now().minus({hour: 24}).toSeconds()
    const timeTo = DateTime.now().plus({hour: 1}).toSeconds()
    let entities = (await Promise.all(routes.map(async route => {
        if(route.route_id === '71') route.route_short_name = 'M10N'
        const resp = await fetch(`${apiURL}/public-origin-departures-by-route-v1/${route.route_short_name}/${timeFrom}/${timeTo}?api_key=${apiKey}`)
        if(!resp.ok) return []
        let vehicles: MegabusVehicles = await resp.json()
        if(vehicles.code !== 0 || vehicles.routes[0].chronological_departures.length === 0) return []

        let latLongs = groupBy(
            lineSegmentQuery.all(route.route_id) as ({stop_id: string} & Position)[],
            ls => ls.stop_id)

        return vehicles.routes[0].chronological_departures.map((dep: ChronologicalDeparture) => {
            if(dep.trip.id.endsWith("S") || dep.trip.id.endsWith("E")) return undefined // positioning move
            if(dep.active_vehicle === null || dep.tracking.is_completed) return undefined

            const depTime = DateTime.fromSeconds(dep.trip.departure_time_unix, {zone: "GMT"})
            const arrTime = DateTime.fromSeconds(dep.trip.arrival_time_unix, {zone: "GMT"})

            const trip = findTrip(depTime, route.route_id, relativeTo(depTime, depTime), relativeTo(depTime, arrTime), dep.trip.departure_location_name, dep.trip.arrival_location_name)
            if(trip === undefined) return undefined

            // out of all line segments for this candidate, find the closest one
            let routeCand = trip.route.split(",")

            let segmentDistances = [...Array(routeCand.length - 1).keys()].map(i => {
                return LineUtil.pointToSegmentDistance(
                    new Point(dep.active_vehicle!.current_wgs84_longitude_degrees, dep.active_vehicle!.current_wgs84_latitude_degrees),
                    new Point(latLongs[routeCand[i]][0].x, latLongs[routeCand[i]][0].y),
                    new Point(latLongs[routeCand[i+1]][0].x, latLongs[routeCand[i+1]][0].y)
                )
            })

            let index = minIndex(segmentDistances) + 1

            return {
                id: dep.trip.id,
                alert: undefined,
                isDeleted: false,
                tripUpdate: undefined,
                vehicle: {
                    trip: {
                        tripId: trip.trip_id,
                        routeId: route.route_id,
                        directionId: -1,
                        startTime: depTime.toSQLTime(),
                        startDate: depTime.toISODate({format: "basic"})!,
                        scheduleRelationship: dep.tracking.is_cancelled ? TripDescriptor_ScheduleRelationship.CANCELED : TripDescriptor_ScheduleRelationship.SCHEDULED
                    },
                    vehicle: undefined,
                    position: {
                        latitude: dep.active_vehicle.current_wgs84_latitude_degrees,
                        longitude: dep.active_vehicle.current_wgs84_longitude_degrees,
                        bearing: dep.active_vehicle.current_forward_azimuth_degrees,
                        odometer: -1,
                        speed: -1
                    },
                    currentStopSequence: Number(trip.seqs.split(",")[index]),
                    stopId: routeCand[index],
                    currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                    timestamp: dep.active_vehicle.last_update_time_unix,
                    congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                    occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
                }
            }
        }).filter(d => d !== undefined) as FeedEntity[]
    }))).flat()
    return { entities }
}

new Feeder(load_coaches).init()