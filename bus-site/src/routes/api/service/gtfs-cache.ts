import {type Alert, FeedMessage} from "./gtfs-realtime";
import type {ServiceData} from "../../../api.type";
import {GET as serviceGet} from "./+server";
import {env} from "$env/dynamic/private";
import {FeedMessageWithAlerts, type StopAlerts} from "./gtfs_protobuf.ts";

/*
 * Realtime data
 */

export let gtfsCache: FeedMessage & StopAlerts = {header: undefined, entity: [], alerts: []}
// Caches /api/service outputs for tracking services to prevent latency from recalculating delay (cleared on GTFS update)
let serviceCache: Record<string, ServiceData> = {}
let alertCache: AlertCaches = {route: {}, stop: {}, trip: {}, agency: {}}

// Download GTFS data and update it every 10s
export async function initGTFS() {
    if(env.GTFS !== "OFF") {
        await downloadGTFS()
        gtfsUpdateLoop()
    }
}

function gtfsUpdateLoop() {
    setTimeout(async () => {
        await downloadGTFS()
        gtfsUpdateLoop()
    }, 10000)
}

// Download GTFS data
export async function downloadGTFS() {
    const gtfsURL = env.GTFS_URL ?? "http://localhost:3000/api/gtfsrt/proto";
    try {
        const gtfsResp = await fetch(gtfsURL)
        if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache // Fail nicely - provide previous cache
        gtfsCache = FeedMessageWithAlerts.decode(new Uint8Array(await gtfsResp.arrayBuffer()))
    } catch (e) {
        console.log("GTFS fetch failed")
        gtfsCache = {header: undefined, entity: [], alerts: []}
    }
    serviceCache = {}
    createAlertCaches()
}

// Locate trip in GTFS cache
export function findRealtimeTrip(tripID: string) {
    return gtfsCache.entity.find(entity => entity.vehicle?.trip?.tripId === tripID)
}

// Return a service in the /api/service cache or fetches it
export async function getRTServiceData(tripID: string): Promise<ServiceData> {
    if(!serviceCache[tripID]) {
        // @ts-ignore
        serviceCache[tripID] = await (await serviceGet({
            url: new URL(`http://localhost/api/service?id=${tripID}`)
        })).json()
    }
    return serviceCache[tripID]
}

type AlertCache = Record<string, Alert[]>
type AlertCaches = {
    route: AlertCache,
    stop: AlertCache,
    trip: AlertCache,
    agency: AlertCache
}
function createAlertCaches() {
    let route: AlertCache = {}
    let stop: AlertCache = {}
    let trip: AlertCache = {}
    let agency: AlertCache = {}
    gtfsCache.alerts.forEach(alert => alert.informedEntity.forEach(entity => {
        if(entity.trip?.tripId) insertIntoCache(trip, entity.trip.tripId, alert)
        if(entity.stopId) insertIntoCache(stop, entity.stopId, alert)
        if(entity.routeId) insertIntoCache(route, entity.routeId, alert)
        if(entity.agencyId && !entity.routeId) insertIntoCache(agency, entity.agencyId, alert)
    }))
    alertCache = {route, stop, trip, agency}
}

function insertIntoCache(cache: AlertCache, id: string, alert: Alert) {
    if(cache[id] === undefined) cache[id] = []
    cache[id].push(alert)
}

export function getStopAlerts(code: string) {
    return alertCache.stop[code] ?? []
}

export function getTripAlerts(trip: string) {
    return alertCache.trip[trip] ?? []
}

export function getRouteAlerts(routeID: string) {
    return alertCache.route[routeID] ?? []
}

export function getAgencyAlerts(agency: string) {
    return alertCache.route[agency] ?? []
}