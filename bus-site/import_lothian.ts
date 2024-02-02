import type {LothianPatterns, LothianRoutes, LothianTimetables} from "./src/api.type";
import {db} from "./src/db";
import {DateTime} from "luxon";
import {allRoutesQuery} from "./import_passenger";
import {relativeTo} from "./src/routes/api/service/realtime_util.ts";

const lothian = 'OP596'
const lothianCountry = 'OP597'
const ecb = 'OP598'
export const lothianOpCodes = [lothian, lothianCountry, ecb]

const routeQueryStmt = db.prepare(
    `SELECT start.departure_time as minss, start.stop_id as startStop, finish.departure_time as maxss, finish.stop_id as finishStop, trips.trip_id
            FROM trips
                LEFT OUTER JOIN main.calendar c on trips.service_id = c.service_id
                LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=:date)
                INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
            WHERE route_id=:route
              AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
              AND NOT (exception_type IS NOT NULL AND exception_type = 2)`
)
const routeQuery = (date: string, route: string) => routeQueryStmt.all(
    {date: Number(date), route, day: DateTime.fromFormat(date, "yyyyMMdd").weekday - 1}) as {minss: number, startStop: string, maxss: number, finishStop: string, trip_id: string}[]

const routeInsert = db.prepare("INSERT INTO polar (gtfs, polar) VALUES (?,?)")
const routeInsertAll = db.transaction((trips: Set<string>, lothianRoute: string) => {
    trips.forEach(trip => {
        routeInsert.run(trip, lothianRoute)
    })
})
const patternInsert = db.prepare("INSERT INTO lothian (pattern, route) VALUES (?,?)")
const patternInsertAll = db.transaction((patterns: string[], route: string) => {
    patterns.forEach((pattern: string) => {
        patternInsert.run(pattern, route)
    })
})

export async function download_route_data() {
    db.exec("DELETE FROM polar WHERE direction IS NULL")
    const routesObj = await (await fetch("https://lothianapi.com/routes")).json() as LothianRoutes
    const currentDate = DateTime.now()

    for(let group of routesObj.groups) {
        console.log(group.name)
        let agencyID = group.id === 'country' ? lothianCountry : group.id === 'eastcoast' ? ecb : lothian

        const gtfsRoutes = allRoutesQuery.all(agencyID) as {route_id: string, route_short_name: string}[]

        for(let route of group.routes) {
            console.log(route.name)
            let routeID = gtfsRoutes.find(gtfs => gtfs.route_short_name === route.name)?.route_id
            if(!routeID) return

            let routePatterns = await (await fetch(`https://lothianapi.com/routePatterns?route_name=${route.name}`)).json() as LothianPatterns
            patternInsertAll(routePatterns.patterns.map(p => p.id), routeID)
            await Promise.all(routePatterns.patterns.map(async pattern => {
                console.log("- " + pattern.id)
                // id=46:627001060800-6280325795
                let allocateds: Set<string> = new Set()
                let timetablePromises = await Promise.allSettled([...new Int8Array(7).keys()].map(async i => {
                    const date = currentDate.plus({days: i})
                    const dateString = date.toISODate({format: "basic"})!
                    let timetable = await (await fetch(`https://lothianapi.com/timetable?route_pattern_id=${pattern.id}&date=${dateString}`)).json() as LothianTimetables
                    const routeTrips = routeQuery(dateString, routeID!)

                    for(let trip of timetable.timetable.trips) {
                        trip.departures = trip.departures.filter(dep => dep.time !== "-")
                        const tripOrigin = trip.departures[0]
                        const tripDest = trip.departures[trip.departures.length - 1]
                        const originTime = DateTime.fromSeconds(tripOrigin.scheduledFor.unixTime)
                        const destTime = DateTime.fromSeconds(tripDest.scheduledFor.unixTime)
                        const originTimeSecs = relativeTo(originTime, originTime)
                        const destTimeSecs = relativeTo(originTime, destTime)
                        const locatedTrip = routeTrips.find(trip => {
                            return trip.startStop === tripOrigin.stopID && trip.finishStop === tripDest.stopID
                                && trip.minss === originTimeSecs && trip.maxss === destTimeSecs
                        })
                        if(locatedTrip) allocateds.add(locatedTrip.trip_id)
                    }
                }))
                timetablePromises.forEach((result, i) => {
                    if(result.status === "rejected") console.log(pattern.id, i, result.reason)
                })
                routeInsertAll(allocateds, pattern.id)
            }))
        }
    }
}

if(process.argv[2] === 'lothian') {
    await download_route_data()
}