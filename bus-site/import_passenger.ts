import {DateTime} from "luxon";
import type {PolarLines, PolarTimetable} from "./src/api.type";
import {dayDiff, relativeTo} from "./src/routes/api/service/realtime_util";
import {db} from "./src/db";
import sourceFile from "./src/routes/api/service/passenger-sources.json" assert {type: "json"}

export const allRoutesQuery = db.prepare("SELECT route_id,route_short_name FROM routes WHERE agency_id=?")
const routeQueryStmt = db.prepare(
    `SELECT start.departure_time as minss, finish.departure_time as maxss, trips.trip_id
            FROM trips
                LEFT OUTER JOIN main.calendar c on trips.service_id = c.service_id
                LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=:date)
                INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
            WHERE route_id=:route
              AND ((start_date <= :date AND end_date >= :date AND (validity & (1 << :day)) <> 0) OR exception_type=1)
              AND NOT (exception_type IS NOT NULL AND exception_type = 2)`
)
const routeQuery = (date: string, route: string) => routeQueryStmt.all({date: Number(date), route, day: DateTime.fromFormat(date, "yyyyMMdd").weekday - 1}) as {minss: number, maxss: number, trip_id: string}[]

// direction+0 and ORDER BY needed to trick SQLite query optimiser into in-memory sorting rather than building an index
const inboundOutbounds = db.prepare(
    `SELECT DISTINCT origin.stop_id as origin, dest.stop_id as dest, direction + 0 AS direction
            FROM polar
                INNER JOIN main.trips t on t.trip_id = polar.gtfs
                INNER JOIN main.stop_times origin on (t.trip_id = origin.trip_id AND origin.stop_sequence=t.min_stop_seq)
                INNER JOIN main.stop_times dest on (t.trip_id = dest.trip_id AND dest.stop_sequence=t.max_stop_seq)
            WHERE t.route_id=? ORDER BY direction`)

const missingDirections = db.prepare(
    `SELECT trips.trip_id, origin.stop_id AS origin, dest.stop_id AS dest FROM trips
                LEFT OUTER JOIN main.polar p on trips.trip_id = p.gtfs
                INNER JOIN main.stop_times origin on (trips.trip_id = origin.trip_id AND origin.stop_sequence=trips.min_stop_seq)
                INNER JOIN main.stop_times dest on (trips.trip_id = dest.trip_id AND dest.stop_sequence=trips.max_stop_seq)
            WHERE route_id=? AND p.direction IS NULL`)

const fixMissingDirection = db.prepare(
    `INSERT INTO polar (gtfs, direction) VALUES (:trip_id, :direction)`
)

type Missing = {
    trip_id: string,
    origin: string,
    dest: string
}

type OriginDestDirection = {
    origin: string,
    dest: string,
    direction: string
}

const directions = ["inbound", "outbound"]

const insert = db.prepare("INSERT OR IGNORE INTO polar (gtfs, polar, direction) VALUES (:gtfs, :polar, :direction)")
export async function downloadRouteDirections() {
    console.log("Downloading route directions for Passenger vehicle detection")
    const currDate = DateTime.now()
    const days = [...new Array(7).keys()].map((i) => currDate.plus({'days': i}))

    db.exec("DELETE FROM polar WHERE direction IS NOT NULL")
    for(let [baseURL, operators] of Object.entries(sourceFile.sources)) {
        const routeNamesResp = await fetch(`${baseURL}/network/lines`)
        if(!routeNamesResp.ok) {
            console.log(`Could not get response for lines of ${baseURL}`)
        }
        const routeNames = await routeNamesResp.json() as PolarLines

        for(let op of operators) {
            const operator = op as keyof typeof sourceFile.operators
            console.log(operator)

            const routes = allRoutesQuery.all(sourceFile.operators[operator].gtfs) as {route_id: string, route_short_name: string}[]
            for(let route of routes) {
                let timetableName = routeNames._embedded["transmodel:line"]
                    .find(l => l.name.toLowerCase() === route.route_short_name.toLowerCase()
                        && l._embedded["transmodel:operator"].code === sourceFile.operators[operator].opCode)?.name
                if(!timetableName) {
                    console.log(`No route exists on web data for ${operator}/${route.route_short_name}`)
                    continue
                }
                console.log(`- ${route.route_short_name} (${timetableName})`)

                let dayPromises = days.flatMap(async day => {
                    const routes = routeQuery(day.toFormat("yyyyMMdd"), route.route_id)
                    return Promise.all(directions.map(async direction => {
                        const lineTimetableResp = await fetch(`${baseURL}/network/operators/${sourceFile.operators[operator].opCode}/lines/${timetableName}/timetables?direction=${direction}&date=${day.toSQLDate()}`)
                        if (!lineTimetableResp.ok) {
                            console.log(`Could not get response for line ${timetableName} of ${operator} (${direction}, ${day.toSQLDate()})`)
                            return []
                        }
                        const lineTimetable = await lineTimetableResp.json() as PolarTimetable

                        return lineTimetable?._embedded?.["timetable:journey"]
                            ?.filter(tj => tj._links["transmodel:line"].name === timetableName)
                            .map(tj => {
                                let oTime = DateTime.fromISO(tj._embedded["timetable:visit"][0].aimedDepartureTime!).set({second: 0, millisecond: 0})
                                let dTime = DateTime.fromISO(tj._embedded["timetable:visit"][tj._embedded["timetable:visit"].length - 1].aimedArrivalTime).set({second: 0, millisecond: 0})
                                let daysDiff = dayDiff(oTime, dTime)
                                if (daysDiff >= 1) dTime = dTime.plus({days: daysDiff})

                                let trip = routes.find(r =>
                                    r.minss === relativeTo(oTime, oTime) && r.maxss === relativeTo(oTime, dTime))
                                if(trip === undefined) {
                                    // Some misses seem to be due to rounding
                                    dTime = dTime.minus({minute: 1})
                                    trip = routes.find(r =>
                                        r.minss === relativeTo(oTime, oTime) && r.maxss === relativeTo(oTime, dTime))
                                }

                                return trip?.trip_id ? {
                                    gtfs: trip.trip_id,
                                    polar: tj.id,
                                    direction: directions.indexOf(direction)
                                } : undefined
                            })
                            .filter(trip => trip !== undefined) ?? []
                    }))
                })
                const linkedTrips = (await Promise.all(dayPromises)).flat(2)
                db.transaction((trips: typeof linkedTrips) => {
                    trips.forEach(trip => insert.run(trip))
                })(linkedTrips)

                // Fill in missing trips

                const dirs = inboundOutbounds.all(route.route_id) as OriginDestDirection[]
                const missings = missingDirections.all(route.route_id) as Missing[]

                const dirFixes = missings.map(m => {
                    let dir = dirs.find(d => d.origin === m.origin && d.dest === m.dest)?.direction
                    if(!dir) {
                        // More lax version of direction finding for part trips - only applied if nothing conflicts
                        let allDirs = [...new Set(dirs.filter(d => d.origin === m.origin || d.dest === m.dest)
                            .map(d => d.direction))]
                        if(allDirs.length === 1) dir = allDirs[0]
                    }
                    return {
                        trip_id: m.trip_id,
                        direction: dir
                    }
                }).filter(d => d.direction !== undefined)

                db.transaction((fixes: typeof dirFixes) => {
                    fixes.forEach(f => fixMissingDirection.run(f))
                })(dirFixes)
            }
        }
    }
}

if(process.argv[2] === 'passenger') {
    await downloadRouteDirections()
}