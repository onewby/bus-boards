import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {operatorRegex, operatorMatches, routeOverrides, routeOverridesPrefixes} from "./operators";
import {DateTime} from "luxon";
import type {ServiceBoard} from "../../../darwin/darwin";
import type {StopDeparture} from "../../../api.type";
import {darwin} from "../darwin";
import {findRealtimeTrip, getRTServiceData} from "../service/gtfs-cache";

// Number of hours to get bus services for
const HOURS_TO_SHOW = 2

export const GET: RequestHandler = async ({url}) => {
    const locality = url.searchParams.get("locality")
    const name = url.searchParams.get("name")
    if(locality == null || name == "") throw error(400, "Invalid query provided.")

    let date = url.searchParams.get("date")
    if(date == null || date == "") date = new Date(Date.now()).toISOString()

    let filterLoc = url.searchParams.get("filterLoc")
    let filterName = url.searchParams.get("filterName")
    let filter = filterLoc !== null && filterName !== null

    let requestedTime = DateTime.fromISO(date, {zone: "Europe/London"})
    if(!requestedTime.isValid) throw error(400, `Invalid date.`)
    let dayName = requestedTime.weekdayLong!.toLowerCase()

    let startTime = requestedTime.minus({hour: 2})
    let endTime = requestedTime.plus({hour: HOURS_TO_SHOW})
    let naiveEndTime = addTimeNaive(startTime.toSQLTime()!, HOURS_TO_SHOW)
    let endDayName = endTime.weekdayLong!.toLowerCase()
    let prevDayName = requestedTime.minus({day: 1}).weekdayLong!.toLowerCase()
    let naiveAdd24Start = addTimeNaive(startTime.toSQLTime()!, 24)
    let naiveAdd24End = addTimeNaive(startTime.toSQLTime()!, 24 + HOURS_TO_SHOW)

    let stop_info: any = db.prepare(
        "SELECT id, name, locality_name, locality as locality_code FROM stops WHERE name=? AND locality=?"
    ).get([name, locality])

    if(stop_info == undefined) throw error(404, "Stop not found.")

    let offset = Math.round(requestedTime.diffNow("minutes").minutes)

    let stance_info: any[] = db.prepare(
        "SELECT code, indicator, street, crs, lat, long FROM stances WHERE stop=?"
    ).all(stop_info['id'])

    const stationPromises = Math.abs(offset) <= 120 ?
        [...new Set(stance_info.filter(stance => stance.crs).map(stance => stance.crs))]
        .map(crs => darwin.getDepartureBoard({crs: crs, numRows: 150, timeOffset: offset})
        .catch((_) => {
            let board: ServiceBoard = {
                generatedAt: "",
                crs: crs,
                locationName: stop_info.name,
                platformAvailable: false
            }
            return board
        })) : []

    stance_info.forEach(stance => {
        if(!stance.indicator) stance.indicator = ""
        delete stance.crs
    })
    stance_info.sort((a, b) => a.indicator.localeCompare(b.indicator))

    let stances = stance_info.map(s => s['code'])

    // To select stop_times for all stances within a stop as part of 'WHERE stop_id IN (${params})'
    let params = stances.map(_ => "?").join(", ")

    let dayStmt = db.prepare(stopTimesStmt(dayName, prevDayName, params, undefined, filter))
    let nextDayStmt = db.prepare(stopTimesStmt(endDayName, dayName, params, 1, filter))
    let stop_times: StopDeparture[]
    // If we go past midnight, we need to handle this in SQL
    if(startTime.hour > endTime.hour) {
        // Get yesterday's buses after midnight going into the morning
        stop_times = dayStmt.all(stances, {date: fmtDate(startTime.minus({day: 1})), start: naiveAdd24Start, end: naiveAdd24End, filterName, filterLoc})
            .map(mapWithTimestamp(-1))
        // and add them to today's buses - first everything going from the day into potentially the next morning
        stop_times = stop_times.concat(dayStmt.all(stances, {date: fmtDate(startTime), start: startTime.toSQLTime(), end: naiveEndTime, filterName, filterLoc})
            .map(mapWithTimestamp()))
        // and anything in the morning registered on the next day
        stop_times = stop_times.concat(nextDayStmt.all(stances, {date: fmtDate(endTime), start: "00:00:00", end: endTime.toSQLTime(), filterName, filterLoc})
            .map(mapWithTimestamp()))
    } else {
        // Get yesterday's buses after midnight going into the morning
        stop_times = dayStmt.all(stances, {date: fmtDate(startTime.minus({day: 1})), start: naiveAdd24Start, end: naiveAdd24End, filterName, filterLoc})
            .map(mapWithTimestamp(-1))
        // and add them to today's buses
        stop_times = stop_times.concat(dayStmt.all(stances, {date: fmtDate(startTime), start: startTime.toSQLTime(), end: endTime.toSQLTime(), filterName, filterLoc}))
            .map(mapWithTimestamp())
    }
    stop_times.forEach(time => time['departure_time'] = modTime(time['departure_time']))
    stop_times.forEach(time => time.type = "bus")

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
        _timestamp: toLuxon(service.std ?? service.sta!)
                    .plus({day: fromAfternoon && (service.std ?? service.sta!)[0] === "0" ? 1 : 0})
    }))

    stop_times = stop_times.concat(trainTimes)
    stop_times.sort((a, b) => {
        let diff = a._timestamp!.diff(b._timestamp!, "millisecond")
        return diff.milliseconds > 0 ? 1 : diff.milliseconds < 0 ? -1 : 0
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

    return json({
        "stop": stop_info,
        "stances": stance_info,
        "times": stop_times
    })
}

const fmtDate = (date: DateTime) => Number(date.toFormat("yyyyMMdd"))
const addTimeNaive = (time: string, add: number) => (Number(time.substring(0, 2)) + add).toString().padStart(2, "0") + time.substring(2, time.length)
const modTime = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

// Not really a use for map, but it is helpful for concise syntax
const mapWithTimestamp = (addDays = 0) => {
    return (dep: StopDeparture) => {
        dep._timestamp = toLuxon(dep.departure_time, addDays)
        return dep
    }
}

const toLuxon = (time: string, addDays = 0) => {
    let hrs = Number(time.substring(0, 2))
    addDays += Math.floor(hrs / 24)
    return DateTime.fromSQL(modTime(time)).plus({day: addDays})
}

const stopTimesStmt = (dayName: string, prevDayName: string, params: string, addDay: (-1|0|1) = 0, filter = false) =>
    `SELECT stop_times.trip_id,coalesce(stop_headsign,t.trip_headsign) as trip_headsign,
                    ${addDay == 1 ? "(printf('%02d', (substring(departure_time, 0, 3) + 24)) || substring(departure_time, 3)) as departure_time"
                            : addDay == -1 ? "(printf('%02d', (substring(departure_time, 0, 3) - 24)) || substring(departure_time, 3)) as departure_time"
                            : "departure_time"},
                    s.indicator,r.route_short_name,a.agency_id as operator_id,a.agency_name as operator_name,stop_sequence as seq
                FROM stop_times
                    INNER JOIN trips t on stop_times.trip_id = t.trip_id
                    INNER JOIN stances s ON stop_times.stop_id = s.code
                    INNER JOIN routes r on r.route_id = t.route_id
                    INNER JOIN main.agency a on r.agency_id = a.agency_id
                WHERE
                    stop_id IN (${params}) AND
                    stop_times.stop_sequence <> t.max_stop_seq AND
                    departure_time IS NOT NULL AND
                    ((EXISTS (SELECT 1 FROM calendar WHERE calendar.service_id = t.service_id AND start_date <= :date AND end_date >= :date AND ${dayName}=1)
                        AND NOT EXISTS (SELECT 1 FROM main.calendar_dates WHERE calendar_dates.service_id = t.service_id AND date = :date AND exception_type=2))
                        OR (EXISTS (SELECT 1 FROM main.calendar_dates WHERE calendar_dates.service_id = t.service_id AND date = :date AND exception_type=1)))
                    AND departure_time >= :start AND departure_time <= :end
                    AND pickup_type <> 1
                    ${filter ? "AND EXISTS (SELECT stop_sequence AS inner_seq FROM stop_times WHERE trip_id=t.trip_id AND inner_seq > seq AND stop_id IN (SELECT code FROM stances WHERE stop=(SELECT id FROM stops WHERE locality=:filterLoc AND name=:filterName)))" : ""}
                ORDER BY departure_time`

const isNum = (c: string) => c >= '0' && c <= '9'