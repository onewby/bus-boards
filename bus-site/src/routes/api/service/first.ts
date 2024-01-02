import type {FirstVehicles, FirstWebSocketInfo, Member} from "../../../api.type";
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
import {readFileSync, writeFileSync} from "fs";
import {existsSync} from "node:fs";

// Configure in first.env
const apiKey = readFileSync(new URL("../../../../first.env", import.meta.url), "utf-8")

const operators: Record<string, string> = {
    "FGLA": "OP584",
    "FABD": "OP511"
}

const bounds: Record<string, Region> = {
    "FGLA": [55.7329, -4.7626, 56.0203, -3.8404],
    "FABD": [57.0891, -2.2834, 57.2250, -2.0376]
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

let activeWS: WebSocket | undefined = undefined
let backoff = 1
let regions: Region[] = existsSync("first-regions.json") ? JSON.parse(readFileSync("first-regions.json", "utf-8")) : Object.values(bounds)
const [MIN_LAT, MIN_LON, MAX_LAT, MAX_LON] = [0, 1, 2, 3]
const [LON, LAT] = [0, 1]

type BufferLike = Parameters<WebSocket["send"]>[0]
type RPCRequest = {
    jsonrpc: "2.0",
    id?: string,
    method: string,
    params: any,
    error?: any
}
type Region = [number, number, number, number]

export async function initialise_first() {
    if(activeWS && activeWS.readyState < WebSocket.CLOSING) activeWS.close()

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
        },
        timeout: 10,
    })

    let resolveOuter: () => void
    let promise: Promise<void> = new Promise((resolve) => {resolveOuter = resolve})
    ws.once("open", async () => {
        backoff = 1
        console.log("Connected to FirstBus streaming API")
        resolveOuter()
    })

    // Restart if connection is broken
    ws.on("error", () => {
        backoff = Math.max(backoff * 2, 32)
        let timeout = (backoff + backoff * Math.random()) * 1000
        console.error(`FirstBus websocket was broken, attempting to reconnect (timeout=${timeout}ms)`)
        setTimeout(initialise_first, timeout)
    })

    activeWS = ws
    return promise
}

function generateRequest(region: [number, number, number, number]): RPCRequest {
    return {
        "jsonrpc": "2.0",
        "id": randomUUID(),
        "method": "configuration",
        "params": {
            "min_lon": region[MIN_LON],
            "max_lon": region[MAX_LON],
            "min_lat": region[MIN_LAT],
            "max_lat": region[MAX_LAT]
        }
    }
}

async function get_vehicles() {
    if(!activeWS || activeWS.readyState !== WebSocket.OPEN) {
        console.error("Attempted to get FirstBus vehicles but the WebSocket was disconnected")
        if(activeWS?.readyState === WebSocket.CLOSED) {
            console.log("Attempting to reconnect")
            await initialise_first()
        }
        return []
    }
    let currentRegion = 0
    let vehicles: Member[][] = regions.map(_ => [])
    while(currentRegion < regions.length) {
        let resp: FirstVehicles = await sendAndReceive(activeWS, generateRequest(regions[currentRegion]))
        if(resp.params.resource.member.length >= 50) {
            // Divide into more regions if there are more than 50 buses in one region
            let newRegions = await divideRegions(activeWS, regions[currentRegion])
            // remove existing region, append new ones
            regions.splice(currentRegion, 1, ...newRegions[0])
            vehicles.splice(currentRegion, 1, ...newRegions[1])
            currentRegion += newRegions.length
            writeFileSync("first-regions.json", JSON.stringify(regions))
        } else {
            vehicles[currentRegion] = resp.params.resource.member
            currentRegion++
        }
    }
    return vehicles.flat()
}

export async function load_first_vehicles(): Promise<FeedEntity[]> {
    return (await get_vehicles()).map(vehicle => {
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
                    startDate: vehicle.stops[0].date.replaceAll("-", ""),
                    scheduleRelationship: TripDescriptor_ScheduleRelationship.SCHEDULED
                },
                vehicle: undefined,
                position: {
                    latitude: vehicle.status.location.coordinates[LAT],
                    longitude: vehicle.status.location.coordinates[LON],
                    bearing: vehicle.status.bearing,
                    odometer: -1,
                    speed: -1
                },
                currentStopSequence: vehicle.status.stops_index.value,
                stopId: vehicle.stops[vehicle.status.stops_index.value].atcocode,
                currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                timestamp: DateTime.fromISO(vehicle.status.recorded_at_time).toSeconds(),
                congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
            }
        }
    }).filter(v => v !== undefined) as FeedEntity[]
}

async function divideRegions(ws: WebSocket, initialRegion: Region): Promise<[Region[], Member[][]]> {
    let regionQueue = [initialRegion] // min lat lon, max lat lon
    let finalRegions: Region[] = []
    let vehiclesUpdate: Member[][] = []
    let region
    while (region = regionQueue.pop()) {
        let resp: FirstVehicles = await sendAndReceive(ws, generateRequest(region))
        let height = region[MAX_LAT] - region[MIN_LAT]
        let width = region[MAX_LON] - region[MIN_LON]

        if(resp.params.resource.member.length >= 50) {
            let latMiddle = median(resp.params.resource.member.map(v => v.status.location.coordinates[LAT]))
            let lonMiddle = median(resp.params.resource.member.map(v => v.status.location.coordinates[LON]))

            if(Math.abs(0.5-(Math.abs(latMiddle - region[MIN_LAT])/height)) < Math.abs(0.5-(Math.abs(lonMiddle - region[MIN_LON])/width))) {
                // Lat is more central
                regionQueue.push(
                    [region[MIN_LAT], region[MIN_LON], latMiddle, region[MAX_LON]],
                    [latMiddle, region[MIN_LON], region[MAX_LAT], region[MAX_LON]]
                )
            } else {
                regionQueue.push(
                    [region[MIN_LAT], region[MIN_LON], region[MAX_LAT], lonMiddle],
                    [region[MIN_LAT], lonMiddle, region[MAX_LAT], region[MAX_LON]]
                )
            }
        } else {
            finalRegions.push(region)
            vehiclesUpdate.push(resp.params.resource.member)
        }
    }
    return [finalRegions, vehiclesUpdate]
}

function median<T>(arr: T[], compareFn?: (a: T, b: T) => number) {
    let m = arr.sort(compareFn)
    return m[Math.floor(m.length / 2)]
}

function sendAndReceive(ws: WebSocket, request: RPCRequest): Promise<RPCRequest> {
    return new Promise((resolve, reject) => {
        let nextMessage = false
        let listener: (data: BufferLike) => void
        let cancel = setTimeout(() => {
            ws.removeListener("message", listener)
            reject("Request timed out");
        }, 10000)
        listener = (data: BufferLike) => {
            let response: RPCRequest = JSON.parse(data.toString())
            if(response.id === request.id) {
                nextMessage = true
                if(response.error) {
                    clearTimeout(cancel)
                    ws.removeListener("message", listener)
                    reject(request.error.data)
                }
            } else if(response.method === "update" && nextMessage) {
                clearTimeout(cancel)
                ws.removeListener("message", listener)
                resolve(response)
            }
        }
        ws.send(JSON.stringify(request))
        ws.on("message", listener)
    })
}