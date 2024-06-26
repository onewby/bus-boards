import {DateTime} from "luxon";
import type {FeedEntity} from "../routes/api/service/gtfs-realtime";
import {
    TripDescriptor_ScheduleRelationship,
    VehiclePosition_CongestionLevel,
    VehiclePosition_OccupancyStatus,
    VehiclePosition_VehicleStopStatus
} from "../routes/api/service/gtfs-realtime";
import {db} from "../db";
import {format_gtfs_date, format_gtfs_time, notNull, relativeTo, ZERO_DAY} from "../routes/api/service/realtime_util";
import {type DownloadResponse, Feeder} from "./feeder";

type StagecoachData = {
    "services": StagecoachService[]
}

type TripStop = {
    trip_id: string,
    route_id: string,
    stop_seq: number,
    stop_id: string
}

type StagecoachService = {
    // Fleet number (number)
    fn: number,
    // Update time (timestamp)
    ut: DateTime,
    // Regional NOC
    oc: string,
    // Service (line) number
    sn: string,
    // Direction
    dn: string,
    // Service ID
    sd: string,
    // Local operator code
    so: string,
    // Service name
    sr: string,
    // Cancelled (boolean)
    cd: boolean,
    // Latitude (number)
    la: number,
    // Longitude (number)
    lo: number,
    // Heading (number)
    hg: number,
    // "Calculated heading" (number)
    cg: number,
    // destination
    dd: string,
    // Origin code
    or: string,
    // Origin name
    on: string,
    // Next stop code
    nr: string,
    // Next stop name
    nn: string,
    // Final stop code
    fr: string,
    // Final stop name
    fs: string,
    // Origin std (timestamp)
    ao: DateTime,
    // Origin etd (timestamp)
    eo: DateTime,
    // Next stop sta (timestamp)
    an: DateTime,
    // Next stop eta (timestamp)
    en: DateTime,
    // Next stop std (timestamp)
    ax: DateTime,
    // Next stop etd (timestamp)
    ex: DateTime,
    // Final stop sta (timestamp)
    af: DateTime,
    // Final stop eta (timestamp)
    ef: DateTime,
    // Final stop longitude (number)
    sg: number,
    // Final stop latitude (number)
    sa: number,
    // KML URL
    ku: string,
    // Trip ID
    td: string,
    // Previous stop code
    pr: string,
    // Current stop on route
    cs: string,
    // Next stop on route
    ns: string,
    // Journey completed heuristic
    jc: boolean,
    // "RAG"
    rg: string
}

const SC_OPERATORS = ["SCFI", "SBLB", "SCHI", "STWS"] as const

const SC_LOCAL_OPERATORS: Record<string, string> = {
    "SIF": "OP550",
    "SPH": "OP557",
    "STY": "OP517",
    "BB": "OP512",
    "HI": "OP1024",
    "OC": "OP603",
    "SGL": "OP582",
    "SWB": "OP537"
}

const LAT = "la"
const LON = "lo"
const HEADING = "hg"
const ORIGIN_STD = "ao"
const CANCELLED = "cd"
const UPDATE_TIME = "ut"
const NEXT_STD = "ax"
const TRIP_ID = "td"
const NEXT_STOP_CODE = "ns"
const PREV_STOP_CODE = "pr"
const ROUTE_NUMBER = "sn"
const LOCAL_OPERATOR = "so"
const JOURNEY_DONE = "jc"

function json_reviver(this: any, key: string, value: string) {
    switch(key) {
        case "fn":
        case "hg":
        case "cg":
        case "la":
        case "lo":
        case "sg":
        case "sa":
            return Number(value)
        case "ut":
        case "ao":
        case "eo":
        case "an":
        case "en":
        case "ax":
        case "ex":
        case "af":
        case "ef":
            return DateTime.fromSeconds(Number(value) / 1000, {zone: "Europe/London"})
        case "cd":
        case "jc":
            return value === "True"
        default:
            return value
    }
}

export async function load_all_stagecoach_data(): Promise<DownloadResponse> {
    return {
        entities: (await Promise.all(SC_OPERATORS.map(load_stagecoach_data))).flat(1)
    }
}

// Load Stagecoach bus data, convert to GTFS representation for use with existing realtime service
async function load_stagecoach_data(operator: typeof SC_OPERATORS[number]): Promise<FeedEntity[]> {
    let resp = await fetch(`https://api.stagecoach-technology.net/vehicle-tracking/v1/vehicles?services=:${operator}:::`)
    if(!resp.ok) return []
    const srcJson: StagecoachData = JSON.parse(await resp.text(), json_reviver)
    // load data here
    return srcJson.services.map((sc): FeedEntity | null => {
        if(sc[JOURNEY_DONE] && !sc[CANCELLED]) return null // journey is done, stop tracking
        let timeWithoutMins = sc[ORIGIN_STD].set({second: 0, millisecond: 0})
        // Locate trip ID from origin std and stop
        const stop: TripStop | undefined = findStop.get({date: Number(timeWithoutMins.toISODate({format: 'basic'})), day: timeWithoutMins.weekday - 1},
            SC_LOCAL_OPERATORS[sc[LOCAL_OPERATOR]], sc[ROUTE_NUMBER], sc[NEXT_STOP_CODE], timeWithoutMins.set(ZERO_DAY).toSeconds() + timeWithoutMins.offset * 60) as TripStop | undefined
        if(!stop) return null
        // Locate current stop from next std and stop
        return {
            id: sc[TRIP_ID],
            alert: undefined,
            isDeleted: false,
            tripUpdate: undefined,
            vehicle: {
                trip: {
                    tripId: stop.trip_id,
                    routeId: stop.route_id,
                    directionId: -1,
                    startTime: format_gtfs_time(sc[ORIGIN_STD]),
                    startDate: format_gtfs_date(sc[ORIGIN_STD]),
                    scheduleRelationship: sc[CANCELLED] ? TripDescriptor_ScheduleRelationship.CANCELED : TripDescriptor_ScheduleRelationship.SCHEDULED
                },
                vehicle: undefined,
                position: {
                    latitude: sc[LAT],
                    longitude: sc[LON],
                    bearing: sc[HEADING],
                    odometer: -1,
                    speed: -1
                },
                currentStopSequence: stop.stop_seq,
                stopId: sc[NEXT_STOP_CODE],
                currentStatus: VehiclePosition_VehicleStopStatus.IN_TRANSIT_TO,
                timestamp: sc[UPDATE_TIME].toMillis() / 1000,
                congestionLevel: VehiclePosition_CongestionLevel.UNRECOGNIZED,
                occupancyStatus: VehiclePosition_OccupancyStatus.UNRECOGNIZED
            }
        }
    }).filter(notNull)
}

const findStop = db.prepare(
`SELECT t.trip_id, r.route_id, stop_times.stop_sequence as stop_seq, stop_times.stop_id FROM stop_times
            INNER JOIN trips t on t.trip_id = stop_times.trip_id
            INNER JOIN main.routes r on t.route_id = r.route_id
            INNER JOIN stop_times origin on (origin.trip_id=t.trip_id AND origin.stop_sequence=t.min_stop_seq)
            LEFT OUTER JOIN main.calendar c on c.service_id = t.service_id
            LEFT OUTER JOIN main.calendar_dates d on (d.service_id = c.service_id AND d.date=:date)
        WHERE agency_id = ? AND route_short_name = ? AND stop_times.stop_id=?
          AND origin.departure_time = ?
          AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
          AND NOT (exception_type IS NOT NULL AND exception_type = 2)`)

new Feeder(load_all_stagecoach_data).init()