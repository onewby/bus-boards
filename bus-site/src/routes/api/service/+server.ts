import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {DateTime, Duration} from "luxon";
import proj4 from "proj4";
import {intercityOperators} from "../stop/operators";
import type {ServiceData, ServiceStopData} from "../../../api.type";
import {cacheService, findRealtimeTrip, getRTServiceData} from "./gtfs-cache";

proj4.defs("EPSG:27700","+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +datum=OSGB36 +units=m +no_defs");

export const GET: RequestHandler = async ({url}) => {
    const id = url.searchParams.get("id")
    if(id === null) throw error(400, "ID not provided.")
    if(getRTServiceData(id)) return json(getRTServiceData(id))
    const service = db.prepare(`SELECT route_short_name as code, trip_headsign as dest, max_stop_seq as mss FROM trips
                                            INNER JOIN main.routes r on r.route_id = trips.route_id
                                            WHERE trip_id=?`).get(id)
    if(service === undefined) throw error(404, "Service not found.")
    const stops = db.prepare(`SELECT stops.id, stops.name, indicator as ind, arrival_time as arr,departure_time as dep, l.name as loc,
                                            timepoint as major, drop_off_type as doo, pickup_type as puo, stances.lat as lat, stances.long as long,
                                            stop_sequence as seq, stops.locality_name AS full_loc
                                        FROM stop_times
                                            INNER JOIN stances on stances.code = stop_times.stop_id
                                            INNER JOIN stops on stops.id = stances.stop
                                            INNER JOIN localities l on l.code = stops.locality
                                        WHERE trip_id=? ORDER BY stop_sequence`).all(id)
    const operator = db.prepare(`SELECT agency_name as name, agency_url as url FROM trips
                                            INNER JOIN main.routes r on r.route_id = trips.route_id
                                            INNER JOIN main.agency a on r.agency_id = a.agency_id
                                            WHERE trip_id = ?`).get(id)
    const shape = db.prepare(`SELECT shape_pt_lat as lat, shape_pt_lon as long
                                        FROM shapes INNER JOIN trips t on shapes.shape_id = t.shape_id
                                        WHERE trip_id=? ORDER BY shape_pt_sequence`).all(id)

    // Better coach listings - show root locality name
    if(intercityOperators.includes(operator.name)) {
        stops.forEach((stop) => {
            if(stop.loc.includes("University") || stop.loc.includes("Airport")) return
            let existingLoc = stop.loc
            stop.loc = stop.full_loc.split(" › ")[0];
            if(stop.name == "Park and Ride" && existingLoc != stop.loc) {
                stop.name = existingLoc + " " + stop.name
            }
        })
    }
    stops.forEach((stop) => delete stop.full_loc)

    // Simplify tram listings - show more akin to trains
    switch(operator.name) {
        case "Edinburgh Trams":
        case "Tyne and Wear Metro":
        case "Metrolink":
        case "SPT Subway":
        case "London Underground (TfL)":
            stops.forEach(stop => stop.ind = "")
        // Fallthrough
        case "West Midlands Metro":
        case "Nottingham Express Transit (Tram)":
        case "London Docklands Light Railway - TfL":
        case "London Tramlink":
            stops.forEach(stop => stop.name = stop.name.replace(suffixes[operator.name], ""))
        // Fallthrough
        case "Stagecoach Supertram":
            stops.forEach(stop => {
                if(stop.name !== "Rail Station") stop.loc = "";
            })
    }

    let route: [number, number][]
    if(shape && shape.length > 0) {
        route = shape.map(s => [s.long, s.lat])
    } else {
        route = stops.map(s => [s.long, s.lat])
    }

    let realtime = undefined
    const trip = await findRealtimeTrip(id)
    if(trip) {
        let currentStop = trip.vehicle?.currentStopSequence
        let currentPos = trip.vehicle?.position
        if(currentStop && currentPos) {
            let currentStopIndex = stops.findIndex(stop => stop.seq === currentStop)
            let pos = getStopPositions.all({seq: currentStop, id: id})
            if(pos.length == 2) {
                const prevBNG = bngToWGS84.inverse({x: pos[0]["long"], y: pos[0]["lat"]})
                const currBNG = bngToWGS84.inverse({x: pos[1]["long"], y: pos[1]["lat"]})
                const posBNG = bngToWGS84.inverse({x: currentPos.longitude, y: currentPos.latitude})
                const pct = findPctBetween(prevBNG, currBNG, posBNG)

                // expected stop = stop closest to current time
                const currentTime = DateTime.now()
                // Calculate delay
                let prevStop: ServiceStopData = stops[currentStopIndex - 1]
                let currStop: ServiceStopData = stops[currentStopIndex]

                let prevDep = DateTime.fromSQL(prevStop.dep)
                let currArr = DateTime.fromSQL(currStop.arr)
                let expectedTime = prevDep.plus(currArr.diff(prevDep).mapUnits(u => isNaN(pct) ? u : u * pct))

                let delay = currentTime.diff(expectedTime)

                let delays = stops.slice(currentStopIndex, stops.length)
                for(let stop of delays) {
                    if(delay.toMillis() >= 1000 * 120) {
                        stop.status = "Exp. " + DateTime.fromSQL(stop.arr ?? stop.dep).plus(delay).toFormat("HH:mm")

                        // Absorb delay in different arr/dep times
                        if(stop.arr && stop.dep) {
                            delay = delay.minus(DateTime.fromSQL(stop.dep).diff(DateTime.fromSQL(stop.arr)))
                            if(delay.toMillis() < 0) delay = Duration.fromMillis(0)
                        }
                    } else {
                        stop.status = "On time"
                    }
                }
                if(stops[currentStopIndex].status !== "On time") stops[currentStopIndex].major = true

                realtime = {
                    stop: currentStopIndex,
                    pos: currentPos,
                    pct: pct
                }
            }
        }
    }

    stops.forEach((stop) => {
        stop.doo = stop.doo === 1
        stop.puo = stop.puo === 1
    })
    delete service.mss

    const data: ServiceData = {
        "service": service,
        "operator": operator,
        "branches": [{
            "dest": service.dest,
            "stops": stops,
            "realtime": realtime,
            "route": route
        }]
    }

    if(realtime) cacheService(id, data)

    return json(data)
}

const suffixes: Record<string, string|RegExp> = {
    "Edinburgh Trams": "",
    "Metrolink": " Metrolink Stop",
    "West Midlands Metro": /\(.*\)/,
    "Nottingham Express Transit (Tram)": " Tram Stop",
    "Tyne and Wear Metro": " (Tyne and Wear Metro Station)",
    "Stagecoach Supertram": / \(S Yorks Supertram\)| \(South Yorkshire Supertram\)/,
    "London Docklands Light Railway - TfL": " DLR Station",
    "London Tramlink": " Tram Stop",
    "SPT Subway": " SPT Subway Station",
    "London Underground (TfL)": " Underground Station"
}

const getStopPositions = db.prepare(`SELECT stop_sequence,long,lat FROM stop_times
                                                INNER JOIN stances on stances.code = stop_times.stop_id
                                                WHERE (stop_sequence=:seq - 1 OR stop_sequence=:seq) AND trip_id=:id`)

const getShape = db.prepare(`SELECT shape_pt_lat as lat, shape_pt_lon as lon FROM shapes
                             WHERE shape_pt_sequence >= (SELECT shape_pt_sequence FROM shapes WHERE shape_pt_lat=:min_lat AND shape_pt_lon=:min_lon)
                                AND shape_pt_sequence <= (SELECT shape_pt_sequence FROM shapes WHERE shape_pt_lat=:max_lat AND shape_pt_lon=:max_lon)
                             ORDER BY shape_pt_sequence`)

type Position = {
    x: number,
    y: number
}

function findNearestLinePoint(s1: Position, s2: Position, point: Position): Position {
    const m = (s2.y - s1.y) / (s2.x - s1.x)
    const c = s1.y - m*s1.x
    const xp = (point.y * m + point.x - m*c) / (m**2 + 1)
    const yp = m*xp + c
    return {x: xp, y: yp}
}

function distanceBetween(s1: Position, s2: Position) {
    return Math.sqrt((s2.x - s1.x) ** 2 + (s2.y - s1.y) ** 2)
}

function findPctBetween(s1: Position, s2: Position, point: Position) {
    const linePoint = findNearestLinePoint(s1, s2, point)
    return Math.max(0, 1 - (distanceBetween(s1, linePoint) / distanceBetween(s1, s2)))
}

const bngToWGS84 = proj4("EPSG:27700", "EPSG:4326")