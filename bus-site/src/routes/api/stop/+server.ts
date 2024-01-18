import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {operatorRegex, operatorMatches, routeOverrides, routeOverridesPrefixes} from "./operators";
import {DateTime} from "luxon";
import type {ServiceBoard} from "../../../darwin/darwin";
import type {StopDeparture} from "../../../api.type";
import {darwin} from "../darwin";
import {findRealtimeTrip, getRTServiceData, getStopAlerts} from "../service/gtfs-cache";

// Number of hours to get bus services for
const HOURS_TO_SHOW = 2

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

    let requestedTime = DateTime.fromISO(date, {zone: "Europe/London"})
    if(!requestedTime.isValid) error(400, `Invalid date.`);
    let dayOrd = requestedTime.weekday - 1

    let startTime = requestedTime.minus({hour: 2})
    let endTime = requestedTime.plus({hour: HOURS_TO_SHOW})
    let naiveEndTime = addTimeNaive(requestedTime.toSQLTime()!, HOURS_TO_SHOW)
    let endDayOrd = endTime.weekday - 1
    let naiveAdd24Start = addTimeNaive(startTime.toSQLTime()!, 24)
    let naiveAdd24End = addTimeNaive(naiveAdd24Start, HOURS_TO_SHOW)
    let startTimeYesterday = startTime.minus({day: 1})
    let startTimeYesterdayOrd = startTime.minus({day: 1}).weekday - 1

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

    let stop_times: StopDeparture[]
    // If we go past midnight, we need to handle this in SQL
    if(startTime.hour > endTime.hour) {
        // Get yesterday's buses after midnight going into the morning
        stop_times = stopTimesStmt2.all({stop: stop_info.id, date: fmtDate(startTimeYesterday), day: startTimeYesterdayOrd, start: naiveAdd24Start, end: naiveAdd24End, addDay: 0, filter: Number(filter), filterName, filterLoc})
            .map(mapWithTimestamp(requestedTime, -1))
        // and add them to today's buses - first everything going from the day into potentially the next morning
        stop_times = stop_times.concat(
            stopTimesStmt2.all({stop: stop_info.id, date: fmtDate(startTime), day: dayOrd, start: startTime.toSQLTime(), end: naiveEndTime, addDay: 0, filter: Number(filter), filterName, filterLoc})
                .map(mapWithTimestamp(requestedTime))
        )
        // and anything in the morning registered on the next day
        stop_times = stop_times.concat(
            stopTimesStmt2.all({stop: stop_info.id, date: fmtDate(endTime), day: endDayOrd, start: "00:00:00", end: endTime.toSQLTime(), addDay: 1, filter: Number(filter), filterName, filterLoc})
                .map(mapWithTimestamp(requestedTime, 1))
        )
    } else {
        // Get yesterday's buses after midnight going into the morning
        stop_times = stopTimesStmt2.all({stop: stop_info.id, date: fmtDate(startTimeYesterday), day: startTimeYesterdayOrd, start: naiveAdd24Start, end: naiveAdd24End, addDay: 0, filter: Number(filter), filterName, filterLoc})
            .map(mapWithTimestamp(requestedTime, -1))
        // and add them to today's buses
        stop_times = stop_times.concat(stopTimesStmt2.all({stop: stop_info.id,  date: fmtDate(startTime), day: dayOrd, start: startTime.toSQLTime(), end: endTime.toSQLTime(), addDay: 0, filter: Number(filter), filterName, filterLoc})
            .map(mapWithTimestamp(requestedTime)))
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
        _timestamp: toLuxon(requestedTime, service.std ?? service.sta!)
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

const fmtDate = (date: DateTime) => Number(date.toFormat("yyyyMMdd"))
const addTimeNaive = (time: string, add: number) => (Number(time.substring(0, 2)) + add).toString().padStart(2, "0") + time.substring(2, time.length)
const modTime = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

// Not really a use for map, but it is helpful for concise syntax
const mapWithTimestamp = (date: DateTime, addDays = 0) => {
    return (dep: any) => {
        dep._timestamp = toLuxon(date, dep.departure_time, addDays)
        return dep as StopDeparture
    }
}

const toLuxon = (date: DateTime, time: string, addDays = 0) => {
    let hrs = Number(time.substring(0, 2))
    addDays += Math.floor(hrs / 24)
    return DateTime.fromSQL(`${date.toSQLDate()} ${modTime(time)}`).plus({day: addDays})
}

const stopTimesStmt2 = db.prepare(
    `SELECT stop_times.trip_id,coalesce(stop_headsign,t.trip_headsign) as trip_headsign,
                iif(:addDay=1, printf('%02d', (substring(departure_time, 0, 3) + 24)) || substring(departure_time, 3),
                    iif(:addDay=-1, printf('%02d', (substring(departure_time, 0, 3) - 24)) || substring(departure_time, 3), departure_time)) as departure_time,
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

const isNum = (c: string) => c >= '0' && c <= '9'