import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {db} from "../../../db";
import {DateTime, Duration} from "luxon";
import proj4 from "proj4";
import {intercityOperators} from "../stop/operators";
import type {ServiceData, StopsQuery, ServiceStopData, StopAlert, ServiceInfo} from "../../../api.type";
import {findRealtimeTrip, getAgencyAlerts, getRouteAlerts, getTripAlerts} from "./gtfs-cache";
import {
    type TranslatedString,
    TripDescriptor_ScheduleRelationship,
    TripUpdate_StopTimeUpdate_ScheduleRelationship
} from "./gtfs-realtime";
import {findPctBetween, sqlToLuxon} from "./realtime_util";
import { LatLng } from "../../../leaflet/geo/LatLng.js";
import polyline from "google-polyline";

proj4.defs("EPSG:27700","+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +datum=OSGB36 +units=m +no_defs");

const serviceQuery = db.prepare(
    `SELECT r.route_id as routeID, route_short_name as code, trip_headsign as dest, max_stop_seq as mss FROM trips
                INNER JOIN main.routes r on r.route_id = trips.route_id
                WHERE trip_id=?`)
const stopsQuery = db.prepare(
    `SELECT stops.name, stops.name as display_name, stops.locality, indicator as ind, arrival_time as arr, 
                departure_time as dep, l.name as loc, timepoint as major, drop_off_type as doo, pickup_type as puo,
                stances.lat as lat, stances.long as long, stop_sequence as seq, stops.locality_name AS full_loc
            FROM stop_times
                INNER JOIN stances on stances.code = stop_times.stop_id
                INNER JOIN stops on stops.id = stances.stop
                INNER JOIN localities l on l.code = stops.locality
            WHERE trip_id=? ORDER BY stop_sequence`)
const operatorQuery = db.prepare(
    `SELECT a.agency_id as id, agency_name as name, COALESCE(website, agency_url) as url FROM trips
                INNER JOIN main.routes r on r.route_id = trips.route_id
                INNER JOIN main.agency a on r.agency_id = a.agency_id
                LEFT OUTER JOIN main.traveline t on a.agency_id = t.agency_id
            WHERE trip_id = ?`)
const shapeQuery = db.prepare(`SELECT polyline FROM shapes INNER JOIN trips t on shapes.shape_id = t.shape_id WHERE trip_id=?`).pluck()

export const GET: RequestHandler = async ({url}) => {
    const id = url.searchParams.get("id")
    if(id === null) error(400, "ID not provided.");

    const service = serviceQuery.get(id) as {routeID?: string, code: string, dest: string, mss: number} | undefined
    if(service === undefined) error(404, "Service not found.")
    const stopObjs = stopsQuery.all(id) as StopsQuery[]
    const stops: ServiceStopData[] = stopObjs.map(obj => ({
        ...obj,
        arr: sqlToLuxon(obj.arr).setZone("GMT").toISOTime({suppressSeconds: true, suppressMilliseconds: true, includeOffset: false})!,
        dep: sqlToLuxon(obj.dep).setZone("GMT").toISOTime({suppressSeconds: true, suppressMilliseconds: true, includeOffset: false})!,
        status: undefined
    }))
    const operator = operatorQuery.get(id) as {id?: string, name: string, url: string}
    const shape = shapeQuery.get(id) as string

    let routeID = service.routeID!
    let agencyID = operator.id!

    // Better coach listings - show root locality name
    if(intercityOperators.includes(operator.name)) {
        stops.forEach((stop) => {
            // @ts-ignore
            if(stop.loc.includes("University") || stop.loc.includes("Airport")) return
            let existingLoc = stop.loc
            // @ts-ignore
            stop.loc = stop.full_loc.split(" › ")[0];
            if((stop.name == "Park and Ride" || stop.name == "Rail Station") && existingLoc != stop.loc) {
                stop.display_name = existingLoc + " " + stop.name
            } else if(stop.loc === "Centenary Square") {
                stop.loc = "Birmingham"
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
    let route: string
    if(shape) {
        route = shape
    } else {
        route = polyline.encode(stops.map(s => [s.lat, s.long]))
    }

    let realtime = undefined
    const trip = findRealtimeTrip(id)
    let cancelled = false
    if(trip) {
        cancelled = trip.vehicle?.trip?.scheduleRelationship === TripDescriptor_ScheduleRelationship.CANCELED
            || trip.tripUpdate?.trip?.scheduleRelationship === TripDescriptor_ScheduleRelationship.CANCELED
        if(cancelled) {
            stops.forEach(stop => stop.status = "Cancelled")
        }
        let currentStop = trip.vehicle?.currentStopSequence
        let currentPos = trip.vehicle?.position

        // Ember (and other better GTFS sources) delay calculation
        if(trip.tripUpdate) {
            const date = DateTime.fromFormat(trip.vehicle?.trip?.startDate ?? DateTime.now().toFormat("yyyyMMdd"), "yyyyMMdd", {zone: "Europe/London"})
            let scheduledTimes = stopObjs.map(stop => {
                return date.plus({second: stop.dep})
            })

            let actualTimes = stops.map((stop, i) => {
                let update = trip.tripUpdate!.stopTimeUpdate.find(stu => stop.seq === stu.stopSequence)
                if(update?.scheduleRelationship === TripUpdate_StopTimeUpdate_ScheduleRelationship.SKIPPED) return undefined
                return update?.departure || update?.arrival ? DateTime.fromSeconds(update.departure?.time ?? update.arrival!.time, {zone: "Europe/London"}) : scheduledTimes[i]
            })
            actualTimes.forEach((stop, i) => {
                if(!stop) {
                    stops[i].status = "Skipped"
                    actualTimes[i] = scheduledTimes[i]
                } else if (actualTimes[i]!.toMillis() - scheduledTimes[i].toMillis() < 60 * 1000) {
                    stops[i].status = "On time"
                } else {
                    stops[i].status = "Exp. " + actualTimes[i]!.toFormat("HH:mm")
                }
            })

            let current = actualTimes.findIndex(time => time! >= DateTime.now())
            let pct = current === 0 || current === -1 ? 0 : actualTimes![current]!.diffNow().toMillis() / actualTimes![current]!.diff(actualTimes![current - 1]!).toMillis()
            for (let i = 0; i < current; i++) stops[i].status = "Dep. " + actualTimes[i]!.toFormat("HH:mm")

            realtime = {
                stop: current,
                pct: pct,
                pos: currentPos
            }

        } else if(currentStop !== undefined && currentPos) {
            let currentStopIndex = stops.findIndex(stop => stop.seq === currentStop)
            let pos: any[] = getStopPositions.all({seq: currentStop, id: id})
            if(pos.length == 2) {
                // Positioning
                const prevBNG = bngToWGS84.inverse({x: pos[0]["long"], y: pos[0]["lat"]})
                const currBNG = bngToWGS84.inverse({x: pos[1]["long"], y: pos[1]["lat"]})
                const posBNG = bngToWGS84.inverse({x: currentPos.longitude, y: currentPos.latitude})
                const pct = findPctBetween(prevBNG, currBNG, posBNG)

                if(!cancelled) {
                    const date = DateTime.fromFormat(trip.vehicle?.trip?.startDate ?? DateTime.now().toFormat("yyyyMMdd"), "yyyyMMdd", {zone: "Europe/London"})
                    const scheduledTimes = stopObjs.map(stop => ({
                        arr: stop.arr ? date.plus({second: stop.arr}) : undefined,
                        dep: stop.dep ? date.plus({second: stop.dep}) : undefined
                    }))

                    // Calculate bus delay per stop
                    let prevStop = scheduledTimes[currentStopIndex - 1]
                    let currStop = scheduledTimes[currentStopIndex]

                    let prevDep = prevStop.dep ?? prevStop.arr!
                    let currArr = currStop.arr ?? currStop.dep!
                    // Get the time that the bus should have been at this position at
                    let expectedTime = prevDep.plus(currArr.diff(prevDep).mapUnits(u => isNaN(pct) ? u : u * pct))

                    // Delay = current time - expected time
                    let delay = DateTime.fromSeconds(trip.vehicle?.timestamp ?? Date.now() / 1000).diff(expectedTime)

                    // Apply delay to all stops past the current stop
                    // (don't show 'Departed' if too close to the last stop - may be a GPS error)
                    let evaluateIndex = Math.max(currentStopIndex - 1, 0);
                    let includeLastStop = new LatLng(stops[evaluateIndex].lat, stops[evaluateIndex].long).distanceTo({lat: currentPos.latitude, lng: currentPos.longitude}) <= 50 ? 1 : 0
                    // Only show Departed 5 mins before departure time
                    for (let i = 0; i < stops.length; i++) {
                        let scheduledDep = scheduledTimes[i].dep ?? scheduledTimes[i].arr!
                        let delayedTime = (scheduledTimes[i].arr ?? scheduledTimes[i].dep!).plus(delay)

                        if(i < currentStopIndex - includeLastStop && scheduledDep.diffNow("seconds").get("seconds") < 120) {
                            stops[i].status = "Departed"
                            continue
                        }

                        if(delay.toMillis() >= 1000 * 120 || delay.toMillis() <= -1000 * 60) {
                            if(scheduledDep.minute === delayedTime.minute) {
                                stops[i].status = "On time"
                            } else {
                                stops[i].status = "Exp. " + delayedTime.toFormat("HH:mm")
                            }

                            // Absorb delay in longer layovers
                            if(scheduledTimes[i].arr && scheduledTimes[i].dep) {
                                try {
                                    delay = delay.minus(scheduledTimes[i].dep!.diff(scheduledTimes[i].arr!))
                                } catch(e) {}
                                if(delay.toMillis() < 0) {
                                    delay = Duration.fromMillis(0)
                                    stops[i].status = "On time"
                                }
                            }
                        } else {
                            stops[i].status = "On time"
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

    let alerts: StopAlert[] = [...getTripAlerts(id), ...getRouteAlerts(routeID), ...getAgencyAlerts(agencyID)].map(alert => ({
        header: findBestMatch(alert.headerText),
        description: findBestMatch(alert.descriptionText),
        url: findBestMatch(alert.url)
    }))

    const serviceObj: ServiceInfo = {
        code: service!.code,
        dest: service!.dest,
        cancelled
    }

    const data: ServiceData = {
        "service": serviceObj,
        "operator": operator,
        "branches": [{
            "dest": service.dest,
            "stops": stops,
            "realtime": realtime,
            "route": route
        }],
        "alerts": alerts
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

const bngToWGS84 = proj4("EPSG:27700", "EPSG:4326")

const findBestMatch = (str?: TranslatedString) => str ? str.translation.find(t => t.language == "en")?.text ?? str.translation[0].text! : undefined