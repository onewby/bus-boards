import {DateTime} from "luxon";
import type {PolarLines, PolarTimetable} from "./src/api.type";
import {format_gtfs_time} from "./src/routes/api/service/realtime_util.js";
import {db} from "./src/db.js";
import sourceFile from "./src/routes/api/service/passenger-sources.json" assert {type: "json"}

const allRoutesQuery = db.prepare("SELECT route_id,route_short_name FROM routes WHERE agency_id=?")
const routeQuery = (date: string, route: string) => db.prepare(
    `SELECT start.departure_time as minss, finish.departure_time as maxss, trips.trip_id
                FROM trips
                    LEFT OUTER JOIN main.calendar c on trips.service_id = c.service_id
                    LEFT OUTER JOIN main.calendar_dates d on (c.service_id = d.service_id AND d.date=20231220)
                    INNER JOIN main.stop_times start on (start.trip_id=trips.trip_id AND start.stop_sequence=trips.min_stop_seq)
                    INNER JOIN main.stop_times finish on (finish.trip_id=trips.trip_id AND finish.stop_sequence=trips.max_stop_seq)
                WHERE route_id=:route
                  AND ((start_date <= :date AND end_date >= :date AND ${DateTime.fromFormat(date, "yyyyMMdd").weekdayLong!.toLowerCase()}=1) OR exception_type=1)
                  AND NOT (exception_type IS NOT NULL AND exception_type = 2)`
).all({date: Number(date), route}) as {minss: string, maxss: string, trip_id: string}[]

const directions = ["inbound", "outbound"]

const insert = db.prepare("INSERT OR IGNORE INTO polar (gtfs, polar, direction) VALUES (:gtfs, :polar, :direction)")
export async function downloadRouteDirections() {
    console.log("Downloading route directions for Passenger vehicle detection")
    const currDate = DateTime.fromISO("2024-01-08T10:42:07+0000") // temporary until after holidays
    const days = [...new Array(7).keys()].map((i) => currDate.plus({'days': i}))

    db.exec("DELETE FROM polar")
    for(let [baseURL, operators] of Object.entries(sourceFile.sources)) {
        const routeNamesResp = await fetch(`${baseURL}/network/lines`)
        if(!routeNamesResp.ok) {
            console.log(`Could not get response for lines of ${baseURL}`)
        }
        const routeNames: PolarLines = await routeNamesResp.json()

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

                let dayPromises = days.map(async day => {
                    const routes = routeQuery(day.toFormat("yyyyMMdd"), route.route_id)
                    const trips = []
                    for(let direction of directions) {
                        const lineTimetableResp = await fetch(`${baseURL}/network/operators/${sourceFile.operators[operator].opCode}/lines/${timetableName}/timetables?direction=${direction}&date=${day.toSQLDate()}`)
                        if (!lineTimetableResp.ok) {
                            console.log(`Could not get response for line ${timetableName} of ${operator} (${direction}, ${day.toSQLDate()})`)
                            continue
                        }
                        const lineTimetable: PolarTimetable = await lineTimetableResp.json()

                        trips.push(...(lineTimetable?._embedded?.["timetable:journey"]
                            ?.filter(tj => tj._links["transmodel:line"].name === route.route_short_name)
                            .map(tj => {
                                let oTime = DateTime.fromISO(tj._embedded["timetable:visit"][0].aimedDepartureTime!)
                                let dTime = DateTime.fromISO(tj._embedded["timetable:visit"][tj._embedded["timetable:visit"].length - 1].aimedArrivalTime)
                                let oTimeStr = format_gtfs_time(oTime)
                                let dTimeStr = format_gtfs_time(dTime)
                                let daysDiff = dTime.diff(oTime, ['days', 'hours']).get("days")
                                if (daysDiff >= 1) {
                                    dTimeStr = addTimeNaive(dTimeStr, 24 * daysDiff)
                                }

                                let trip = routes.find(r =>
                                    r.minss === oTimeStr && r.maxss === dTimeStr)

                                return trip?.trip_id ? {
                                    gtfs: trip.trip_id,
                                    polar: tj.id,
                                    direction: directions.indexOf(direction)
                                } : undefined
                            })
                            .filter(trip => trip !== undefined) ?? []))
                    }
                    return trips
                })
                const linkedTrips = (await Promise.all(dayPromises)).flat()
                db.transaction((trips: typeof linkedTrips) => {
                    trips.forEach(trip => {
                        insert.run(trip)
                    })
                })(linkedTrips)
            }
        }
    }
}

const addTimeNaive = (time: string, add: number) => (Number(time.substring(0, 2)) + add).toString().padStart(2, "0") + time.substring(2, time.length)
