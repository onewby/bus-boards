import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {DateTime, Duration} from "luxon";
import proj4 from "proj4";
import {intercityOperators} from "../stop/operators";
import type {ServiceData, ServiceStopData} from "../../../api.type";
import {findRealtimeTrip} from "./gtfs-cache";
import {TripDescriptor_ScheduleRelationship} from "./gtfs-realtime";
import {findPctBetween} from "./realtime_util";

proj4.defs("EPSG:27700","+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +datum=OSGB36 +units=m +no_defs");

export const GET: RequestHandler = async ({url}) => {
    const id = url.searchParams.get("id")
    if(id === null) error(400, "ID not provided.");
    const service: any = db.prepare(`SELECT route_short_name as code, trip_headsign as dest, max_stop_seq as mss FROM trips
                                            INNER JOIN main.routes r on r.route_id = trips.route_id
                                            WHERE trip_id=?`).get(id)
    if(service === undefined) error(404, "Service not found.");
    const stops: ServiceStopData[] = db.prepare(`SELECT stops.name, stops.name as display_name, stops.locality, indicator as ind, arrival_time as arr,departure_time as dep, l.name as loc,
                                            timepoint as major, drop_off_type as doo, pickup_type as puo, stances.lat as lat, stances.long as long,
                                            stop_sequence as seq, stops.locality_name AS full_loc
                                        FROM stop_times
                                            INNER JOIN stances on stances.code = stop_times.stop_id
                                            INNER JOIN stops on stops.id = stances.stop
                                            INNER JOIN localities l on l.code = stops.locality
                                        WHERE trip_id=? ORDER BY stop_sequence`).all(id) as ServiceStopData[]
    const operator: any = db.prepare(`SELECT agency_name as name, agency_url as url FROM trips
                                            INNER JOIN main.routes r on r.route_id = trips.route_id
                                            INNER JOIN main.agency a on r.agency_id = a.agency_id
                                            WHERE trip_id = ?`).get(id)
    const shape: any[] = db.prepare(`SELECT shape_pt_lat as lat, shape_pt_lon as long
                                        FROM shapes INNER JOIN trips t on shapes.shape_id = t.shape_id
                                        WHERE trip_id=? ORDER BY shape_pt_sequence`).all(id)

    // Better coach listings - show root locality name
    if(intercityOperators.includes(operator.name)) {
        stops.forEach((stop) => {
            // @ts-ignore
            if(stop.loc.includes("University") || stop.loc.includes("Airport")) return
            let existingLoc = stop.loc
            // @ts-ignore
            stop.loc = stop.full_loc.split(" › ")[0];
            if(stop.name == "Park and Ride" && existingLoc != stop.loc) {
                stop.display_name = existingLoc + " " + stop.name
            }
        })
    }
    // @ts-ignore
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
            stops.forEach(stop => stop.display_name = stop.name.replace(suffixes[operator.name], ""))
        // Fallthrough
        case "Stagecoach Supertram":
            stops.forEach(stop => {
                if(stop.name !== "Rail Station") stop.loc = "";
            })
    }

    // Route line shape for map
    let route: [number, number][]
    if(shape && shape.length > 0) {
        route = shape.map(s => [s.long, s.lat])
    } else {
        route = stops.map(s => [s.long, s.lat])
    }

    let realtime = undefined
    const trip = findRealtimeTrip(id)
    service.cancelled = false
    if(trip) {
        service.cancelled = trip.vehicle?.trip?.scheduleRelationship === TripDescriptor_ScheduleRelationship.CANCELED
        if(service.cancelled) {
            stops.forEach(stop => stop.status = "Cancelled")
        }
        let currentStop = trip.vehicle?.currentStopSequence
        let currentPos = trip.vehicle?.position
        if(currentStop !== undefined && currentPos) {
            let currentStopIndex = stops.findIndex(stop => stop.seq === currentStop)
            let pos: any[] = getStopPositions.all({seq: currentStop, id: id})
            if(pos.length == 2) {
                // Positioning
                const prevBNG = bngToWGS84.inverse({x: pos[0]["long"], y: pos[0]["lat"]})
                const currBNG = bngToWGS84.inverse({x: pos[1]["long"], y: pos[1]["lat"]})
                const posBNG = bngToWGS84.inverse({x: currentPos.longitude, y: currentPos.latitude})
                const pct = findPctBetween(prevBNG, currBNG, posBNG)

                if(!service.cancelled) {
                    // Calculate bus delay per stop
                    let prevStop: ServiceStopData = stops[currentStopIndex - 1]
                    let currStop: ServiceStopData = stops[currentStopIndex]

                    let prevDep = DateTime.fromSQL(prevStop.dep)
                    let currArr = DateTime.fromSQL(currStop.arr)
                    // Get the time that the bus should have been at this position at
                    let expectedTime = prevDep.plus(currArr.diff(prevDep).mapUnits(u => isNaN(pct) ? u : u * pct))

                    // Delay = current time - expected time
                    let delay = DateTime.now().diff(expectedTime)

                    // Apply delay to all stops past the current stop
                    let delays = stops.slice(currentStopIndex, stops.length)
                    for(let stop of delays) {
                        if(delay.toMillis() >= 1000 * 120 || delay.toMillis() <= -1000 * 60) {
                            stop.status = "Exp. " + DateTime.fromSQL(stop.arr ?? stop.dep).plus(delay).toFormat("HH:mm")

                            // Absorb delay in longer layovers
                            if(stop.arr && stop.dep) {
                                try {
                                    delay = delay.minus(DateTime.fromSQL(stop.dep).diff(DateTime.fromSQL(stop.arr)))
                                } catch(e) {}
                                if(delay.toMillis() < 0) {
                                    delay = Duration.fromMillis(0)
                                    stop.status = "On time"
                                }
                            }
                        } else {
                            stop.status = "On time"
                        }
                    }
                    // Show current delayed stop in major stops list for context (since previous stops don't show delay, can look on time when delayed)
                    if(stops[currentStopIndex].status !== "On time") stops[currentStopIndex].major = true
                }

                realtime = {
                    stop: currentStopIndex,
                    pos: currentPos,
                    pct: pct
                }
            } else {
                realtime = {
                    stop: -1,
                    pct: 0,
                    pos: currentPos
                }
            }
        }
    }

    // Convert drop off only and put down only to booleans for "doesn't drop off", "doesn't pick up"
    stops.forEach((stop) => {
        // @ts-ignore
        stop.doo = stop.doo === 1
        // @ts-ignore
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

const bngToWGS84 = proj4("EPSG:27700", "EPSG:4326")