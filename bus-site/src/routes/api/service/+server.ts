import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {DateTime} from "luxon";
import {FeedMessage} from "./gtfs-realtime";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import proj4 from "proj4";

proj4.defs("EPSG:27700","+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +datum=OSGB36 +units=m +no_defs");

export const GET: RequestHandler = async ({url}) => {
    const id = url.searchParams.get("id")
    if(id === null) throw error(400, "ID not provided.")
    const service = db.prepare(`SELECT route_short_name as code, trip_headsign as dest, max_stop_seq as mss FROM trips
                                            INNER JOIN main.routes r on r.route_id = trips.route_id
                                            WHERE trip_id=?`).get(id)
    if(service === undefined) throw error(404, "Service not found.")
    const stops = db.prepare(`SELECT stops.id, stops.name, indicator as ind, arrival_time as arr,departure_time as dep, l.name as loc,
                                            timepoint as major, drop_off_type as doo, pickup_type as puo, stances.lat as lat, stances.long as long,
                                            stop_sequence as seq
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

    // Simplify tram listings - show more akin to trains
    switch(operator.name) {
        case "Edinburgh Trams":
        case "Tyne and Wear Metro":
        case "Metrolink":
            stops.forEach(stop => stop.ind = "")
        // Fallthrough
        case "West Midlands Metro":
        case "Nottingham Express Transit (Tram)":
            stops.forEach(stop => stop.name = stop.name.replace(suffixes[operator.name], ""))
        // Fallthrough
        case "Stagecoach Supertram":
            stops.forEach(stop => stop.loc = "")
    }

    let route
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
                const {x: e1, y: n1} = bngToWGS84.inverse({x: pos[0]["long"], y: pos[0]["lat"]})
                const {x: e2, y: n2} = bngToWGS84.inverse({x: pos[1]["long"], y: pos[1]["lat"]})
                const {x: eP, y: nP} = bngToWGS84.inverse({x: currentPos.longitude, y: currentPos.latitude})

                realtime = {
                    stop: currentStopIndex,
                    pos: currentPos,
                    pct: findPctBetween({x: e1, y: n1}, {x: e2, y: n2}, {x: eP, y: nP})
                }
            }
        }
    }

    stops.forEach((stop) => {
        stop.doo = stop.doo === 1
        stop.puo = stop.puo === 1
        delete stop.seq
    })
    delete service.mss
    
    return json({
        "service": service,
        "operator": operator,
        "branches": [{
            "dest": service.dest,
            "stops": stops,
            "realtime": realtime,
            "route": route
        }]
    })
}

const suffixes: Record<string, string|RegExp> = {
    "Edinburgh Trams": "",
    "Metrolink": " (Manchester Metrolink)",
    "West Midlands Metro": /\(.*\)/,
    "Nottingham Express Transit (Tram)": " Tram Stop",
    "Tyne and Wear Metro": " (Tyne and Wear Metro Station)"
}

/*
 * Realtime data
 */

let gtfsCache: FeedMessage = {header: undefined, entity: []}
let lastCacheTime = DateTime.now().minus({minute: 10})

async function getGTFS() {
    if(lastCacheTime.diffNow("seconds").seconds <= -30) {
        const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
        if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache

        const zipReader = new ZipReader(gtfsResp.body)
        let file = (await zipReader.getEntries()).shift()
        if(!file) return gtfsCache

        gtfsCache = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))
        lastCacheTime = DateTime.now()
    }
    return gtfsCache
}

async function findRealtimeTrip(tripID: string) {
    let gtfs = await getGTFS()
    return gtfs.entity.find(entity => entity.vehicle?.trip?.tripId === tripID)
}

const getStopPositions = db.prepare(`SELECT stop_sequence,long,lat FROM stop_times
                                                INNER JOIN stances on stances.code = stop_times.stop_id
                                                WHERE (stop_sequence=:seq OR stop_sequence=:seq + 1) AND trip_id=:id`)

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