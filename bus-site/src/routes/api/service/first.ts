import type {FirstVehicles, FirstWebSocketInfo} from "../../../api.type";
import {WebSocket} from "ws";
import {randomUUID} from "node:crypto";
import {
    type FeedEntity,
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel, VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "./gtfs-realtime.js";
import {db} from "../../../db.js";
import {DateTime} from "luxon";
import {readFileSync} from "fs";

// Configure in first.env
const apiKey = readFileSync(new URL("../../../../first.env", import.meta.url), "utf-8")

const operators: Record<string, string> = {
    "FGLA": "OP584",
    "FABD": "OP511"
}

const tripQuery = (date: DateTime, route: string, op: string, startStop: string, startTime: string) => db.prepare(
    `SELECT trips.trip_id, trips.route_id FROM trips
                INNER JOIN main.routes r on r.route_id = trips.route_id
                INNER JOIN main.stop_times st on (trips.trip_id = st.trip_id AND stop_sequence=min_stop_seq)
                LEFT OUTER JOIN main.calendar c on c.service_id = trips.service_id
                LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
            WHERE route_short_name=:route AND agency_id=:op AND st.stop_id=:startStop AND SUBSTR(st.departure_time, 1, 5)=:startTime
                AND ((start_date <= :date AND end_date >= :date AND ${date.weekdayLong!.toLowerCase()}=1) OR exception_type=1)
                    AND NOT (exception_type IS NOT NULL AND exception_type = 2)`
).get({date: Number(date.toFormat("yyyyMMdd")), route, op, startStop, startTime}) as {trip_id: string, route_id: string}

export async function load_first_vehicles(): Promise<FeedEntity[]> {
    let vehicles = await download_vehicles()
    const updateTime = Date.now() / 1000
    return vehicles.params.resource.member.map(vehicle => {
        let gtfsTrip = tripQuery(DateTime.fromSQL(vehicle.stops[0].date), vehicle.line_name, operators[vehicle.operator], vehicle.origin_atcocode, vehicle.stops[0].time)
        if(gtfsTrip === undefined) return undefined
        return {
            id: vehicle.status.vehicle_id,
            alert: undefined,
            isDeleted: false,
            tripUpdate: undefined,
            vehicle: {
                trip: {
                    tripId: gtfsTrip.trip_id,
                    routeId: gtfsTrip.route_id,
                    directionId: -1,
                    startTime: vehicle.stops[0].time + ":00",
                    startDate: vehicle.stops[0].date,
                    scheduleRelationship: TripDescriptor_ScheduleRelationship.SCHEDULED
                },
                vehicle: undefined,
                position: {
                    latitude: vehicle.status.location.coordinates[1],
                    longitude: vehicle.status.location.coordinates[0],
                    bearing: vehicle.status.bearing,
                    odometer: -1,
                    speed: -1
                },
                currentStopSequence: vehicle.status.stops_index.value,
                stopId: vehicle.stops[vehicle.status.stops_index.value].atcocode,
                currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                timestamp: updateTime,
                congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
            }
        }
    }).filter(v => v !== undefined) as FeedEntity[]
}

async function download_vehicles(): Promise<FirstVehicles> {
    const wsInfoResp = await fetch("https://prod.mobileapi.firstbus.co.uk/api/v2/bus/service/socketInfo", {
        headers: {
            "x-app-key": apiKey
        }
    })
    if(!wsInfoResp.ok) return {jsonrpc: "", method: "", params: {resource: {member: []}}}

    const wsInfo: FirstWebSocketInfo = await wsInfoResp.json()

    const ws = new WebSocket("wss://streaming.bus.first.transportapi.com/", {
        headers: {
            "Authorization": `Bearer ${wsInfo.data["access-token"]}`
        }
    })
    await connect(ws)
    let resp: FirstVehicles = await send(ws, JSON.stringify({
        "jsonrpc": "2.0",
        "id": randomUUID(),
        "method": "configuration",
        "params": {
            "min_lon": -6.877,
            "max_lon": 0,
            "min_lat": 54.636,
            "max_lat": 61.133
        }
    }))
    ws.close()
    return resp
}

function connect(ws: WebSocket, timeout = 10000) {
    return new Promise<void>((resolve, reject) => {
        let timeoutObj = setTimeout(reject, timeout)
        ws.once("open", () => {
            clearTimeout(timeoutObj)
            resolve()
        })
    })
}

function send(ws: WebSocket, data: BufferLike, timeout = 10000): Promise<any> {
    return new Promise((resolve, reject) => {
        let timeoutObj = setTimeout(reject, timeout)
        ws.send(data)
        let listener = (data: BufferLike) => {
            let parsed = JSON.parse(data.toString())
            if(parsed["method"] === "update") {
                if(!listener) ws.removeListener("message", listener)
                clearTimeout(timeoutObj)
                resolve(parsed)
            }
        }
        ws.on("message", listener)
    })
}

type BufferLike = Parameters<WebSocket["send"]>[0]