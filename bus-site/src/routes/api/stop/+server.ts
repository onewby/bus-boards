import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {operatorRegex, operatorMatches, routeOverrides, routeOverridesPrefixes} from "./operators";
import {DateTime} from "luxon";
import type {ServiceBoard} from "../../../darwin/darwin";
import type {StopDeparture} from "../../../api.type";
import {darwin} from "../darwin";
import {findRealtimeTrip, getRTServiceData, getStopAlerts} from "../service/gtfs-cache";
import {FULL_TIME, ZERO_DAY, ZERO_TIME} from "../service/realtime_util.ts";

// Number of hours to get bus services for
const HOURS_TO_SHOW = 2

const stopTimesStmt = db.prepare(
    `SELECT stop_times.trip_id,coalesce(stop_headsign,t.trip_headsign) as trip_headsign, departure_time,
                    s.indicator,r.route_short_name,a.agency_id as operator_id,a.agency_name as operator_name,stop_sequence as seq
                FROM stop_times
                    INNER JOIN trips t on stop_times.trip_id = t.trip_id
                    INNER JOIN stances s ON stop_times.stop_id = s.code
                    INNER JOIN routes r on r.route_id = t.route_id
                    INNER JOIN main.agency a on r.agency_id = a.agency_id
                    LEFT OUTER JOIN main.calendar c on t.service_id = c.service_id
                    LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=:date)
                WHERE
                    s.stop=:stop AND
                    stop_times.stop_sequence <> t.max_stop_seq AND
                    departure_time IS NOT NULL
                    AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
                    AND NOT (exception_type IS NOT NULL AND exception_type = 2)
                    AND departure_time >= :start AND departure_time <= :end
                    AND pickup_type <> 1
                    AND (:filter <> 1 OR EXISTS (SELECT stop_sequence AS inner_seq FROM stop_times WHERE trip_id=t.trip_id AND inner_seq > seq AND stop_id IN (SELECT code FROM stances WHERE stop=(SELECT id FROM stops WHERE locality=:filterLoc AND name=:filterName))))
                ORDER BY departure_time`)
const stopInfoQuery = db.prepare(
    "SELECT id, name, locality_name, locality as locality_code FROM stops WHERE name=? AND locality=?")
type StopInfoQuery = {id: string, name: string, locality_name: string, locality: string}
const stanceInfoQuery = db.prepare(
    "SELECT code, indicator, street, crs, lat, long FROM stances WHERE stop=?")

export const GET: RequestHandler = async ({url}) => {
    const locality = url.searchParams.get("locality")
    const name = url.searchParams.get("name")
    if(locality == null || name == "") error(400, "Invalid query provided.");

    let date = url.searchParams.get("date")
    if(date == null || date == "") date = DateTime.now().set({second: 0, millisecond: 0}).toISO()!

    let filterLoc = url.searchParams.get("filterLoc")
    let filterName = url.searchParams.get("filterName")
    let filter = filterLoc !== null && filterName !== null

    let requestedTime = DateTime.fromISO(date, {zone: "GMT"})
    if(!requestedTime.isValid) error(400, `Invalid date.`);

    let startTime = requestedTime.minus({hour: 2})
    let endTime = requestedTime.plus({hour: HOURS_TO_SHOW})

    let stop_info = stopInfoQuery.get([name, locality]) as StopInfoQuery | undefined
    if(stop_info === undefined) error(404, "Stop not found.")

    let offset = Math.round(requestedTime.diffNow("minutes").minutes)

    let stance_info: any[] = stanceInfoQuery.all(stop_info['id'])

    const stationPromises = Math.abs(offset) <= 120 ?
        [...new Set(stance_info.filter(stance => stance.crs).map(stance => stance.crs))]
        .map(crs => darwin.getDepartureBoard({crs: crs, numRows: 150, timeOffset: offset})
        .catch((_) => {
            let board: ServiceBoard = {
                generatedAt: "",
                crs: crs,
                locationName: stop_info!.name,
                platformAvailable: false
            }
            return board
        })) : []

    stance_info.forEach(stance => {
        if(!stance.indicator) stance.indicator = ""
        delete stance.crs
    })
    stance_info.sort((a, b) => a.indicator.localeCompare(b.indicator))

    let stop_times: StopDeparture[] = []
    if(startTime.day !== requestedTime.day || endTime.day !== requestedTime.day) {
        stop_times = [
            ...getBetween(startTime, startTime.set(FULL_TIME), stop_info.id, Number(filter), filterName, filterLoc),
            ...getBetween(endTime.set(ZERO_TIME), endTime, stop_info.id, Number(filter), filterName, filterLoc)
        ]
    } else {
        // get current day
        stop_times = getBetween(startTime, endTime, stop_info?.id, Number(filter), filterName, filterLoc)
    }

    // Coastliner/Flyer workaround (duplicate services under Coastliner + Flyer names, only Coastliner ones track)
    let replaced: StopDeparture[] = []
    stop_times.filter((time) => (
        time.operator_name === "Coastliner"
        && time.route_short_name.startsWith("A")))
    .forEach((time) => {
        let toReplace = stop_times.find((s2) => s2.operator_name === "Flyer" && s2.route_short_name === time.route_short_name && s2.departure_time === time.departure_time)
        if(toReplace) {
            time.operator_name = "Flyer"
            replaced.push(toReplace)
        }
    })
    stop_times = stop_times.filter((stop) => !replaced.includes(stop))
    // SPT Subway workaround
    stop_times = stop_times.filter((stop) => stop.operator_name !== "SPT Subway" || stop.indicator)

    // Realtime bus info
    // - Find the buses that are currently tracking
    const trackingStops: StopDeparture[] = stop_times.filter(stop => findRealtimeTrip(stop.trip_id))
    await Promise.all(trackingStops.map(async stop => {
        // @ts-ignore
        let serviceData = await getRTServiceData(stop.trip_id)
        if(!serviceData || serviceData.branches.length != 1 || !stop.seq) return;
        // - Update their status
        stop.status = serviceData.branches[0].stops.find(searchingStop => searchingStop.seq === stop.seq)?.status
    }))
    stop_times = stop_times.filter(stop => stop._timestamp! >= requestedTime || stop.status?.startsWith("Exp. ") || stop.status === "Cancelled")

    const stations = await Promise.all(stationPromises)
    const services = stations.filter(board => board.trainServices).flatMap(board => board.trainServices!.service)
    const fromAfternoon = services.length > 0 && (services[0].std ?? services[0].sta!)[0] === "1"
    const trainTimes: StopDeparture[] = services.filter(service => service.operatorCode !== "TW").map(service => ({
        trip_id: service.serviceID,
        trip_headsign: service.destination.location.map(loc => loc.locationName).join(" & "),
        route_short_name: "",
        departure_time: service.std ?? service.sta!,
        indicator: service.platform ? `Platform ${service.platform}` : "Platform TBC",
        operator_name: service.operator,
        operator_id: service.operatorCode,
        colour: "",
        type: "train",
        status: service.etd !== undefined && isNum(service.etd[0]) ? "Exp. " + service.etd : service.etd,
        _timestamp: toLuxon(requestedTime, service.std ?? service.sta!)
                    .plus({day: fromAfternoon && (service.std ?? service.sta!)[0] === "0" ? 1 : 0})
    }))

    stop_times = stop_times.concat(trainTimes)
    stop_times.sort((a, b) => {
        return a._timestamp.toSeconds() - b._timestamp.toSeconds()
    })

    // Determine operator colours
    const agencies: Set<string> = new Set(stop_times.map(time => time.operator_name))
    const colours: Record<string, string> = {}
    agencies.forEach(a => {
        if(a in operatorMatches) return colours[a] = operatorMatches[a]
        for(let [regex, colour] of Object.entries(operatorRegex)) {
            if(a.match(regex) != null) return colours[a] = colour
        }
        return colours[a] = "#777"
    })
    stop_times.forEach(time => time.colour = routeOverrides[time.operator_name]?.[time.route_short_name] ?? routeOverridesPrefixes[time.operator_name]?.[time.route_short_name.match("(.*)[A-Z]")?.[1] ?? time.route_short_name] ?? colours[time.operator_name])

    // Realtime alerts
    let alerts = stance_info.map(stance => getStopAlerts(stance.code)).filter(alerts => alerts !== undefined).flat().map(alert => {
        return {
            header: alert.headerText?.translation[0].text,
            description: alert.descriptionText?.translation[0].text,
            url: alert.url?.translation[0].text
        }
    })

    return json({
        "stop": stop_info,
        "stances": stance_info,
        "times": stop_times,
        "alerts": alerts
    })
}

function getBetween(from: DateTime, to: DateTime, stop: string, filter: number, filterName: string | null, filterLoc: string | null) {
    // get times that run through from the previous day, get times today
    let yesterday = from.minus({day: 1})
    let fromZero = from.set(ZERO_DAY)
    let toZero = to.set(ZERO_DAY)
    return [
        ...(stopTimesStmt.all({
            stop,
            date: fmtDate(yesterday), day: yesterday.weekday - 1,
            start: fromZero.toSeconds() + 86400,
            end: toZero.toSeconds() + 86400,
            filter, filterName, filterLoc}) as StopTimeStmt[])
            .map(mapWithTimestamp(yesterday)),
        ...(stopTimesStmt.all({
            stop,
            date: fmtDate(from), day: from.weekday - 1,
            start: fromZero.toSeconds(),
            end: toZero.toSeconds(),
            filter, filterName, filterLoc}) as StopTimeStmt[])
            .map(mapWithTimestamp(from))
    ]
}

const fmtDate = (date: DateTime) => Number(date.toFormat("yyyyMMdd"))
const modTime = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

// Not really a use for map, but it is helpful for concise syntax
const mapWithTimestamp = (date: DateTime) => {
    let zeroDate = date.set({hour: 0, minute: 0, second: 0, millisecond: 0}).toSeconds()
    return (dep: StopTimeStmt): StopDeparture => {
        let _timestamp = DateTime.fromSeconds(zeroDate + dep.departure_time)
        return {
            ...dep,
            departure_time: _timestamp.toFormat("HH:mm"),
            _timestamp,
            type: "bus",
            colour: "#777"
        }
    }
}

const toLuxon = (date: DateTime, time: string, addDays = 0) => {
    let hrs = Number(time.substring(0, 2))
    addDays += Math.floor(hrs / 24)
    return DateTime.fromSQL(`${date.toSQLDate()} ${modTime(time)}`).plus({day: addDays})
}

const isNum = (c: string) => c >= '0' && c <= '9'

type StopTimeStmt = {trip_id: string, trip_headsign: string, departure_time: number, indicator: string, route_short_name: string, operator_id: string, operator_name: string, seq: number}