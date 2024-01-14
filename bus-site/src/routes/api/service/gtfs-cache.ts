import {type Alert, FeedHeader_Incrementality, FeedMessage} from "./gtfs-realtime";
import type {ServiceData} from "../../../api.type";
import {GET as serviceGet} from "./+server";
import {env} from "$env/dynamic/private";
import {load_gtfs_source} from "../../../realtime/feeder_bods.ts";
import {load_ember} from "../../../realtime/feeder_ember.ts";
import {load_all_stagecoach_data} from "../../../realtime/feeder_stagecoach.ts";
import {load_passenger_sources} from "../../../realtime/feeder_passenger.ts";
import {load_first_vehicles} from "../../../realtime/feeder_first.ts";
import {load_coaches} from "../../../realtime/feeder_coaches.ts";
import {type DownloadResponse, emptyDownloadResponse, type StopAlerts} from "../../../realtime/feeder.ts";
import {merge} from "./realtime_util.js";
import {load_Lothian_vehicles} from "../../../realtime/feeder_lothian.ts";

/*
 * Realtime data
 */

export let gtfsCache: FeedMessage & StopAlerts = {header: undefined, entity: [], alerts: []}
// Caches /api/service outputs for tracking services to prevent latency from recalculating delay (cleared on GTFS update)
let serviceCache: Record<string, ServiceData> = {}
let alertCache: AlertCaches = {route: {}, stop: {}, trip: {}, agency: {}}

// Download GTFS data and update it every 10s
export async function initGTFS() {
    await downloadGTFS()
    gtfsUpdateLoop()
}

function gtfsUpdateLoop() {
    setTimeout(async () => {
        await downloadGTFS()
        gtfsUpdateLoop()
    }, 10000)
}

// Download GTFS data
export async function downloadGTFS() {
    try {
        if((env.GTFS ?? 'disabled') === 'microservice') {
            const gtfsResp = await fetch(env.GTFS_URL ?? "http://localhost:3948")
            if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache // Fail nicely - provide previous cache
            gtfsCache = await gtfsResp.json()
        } else if((env.GTFS ?? 'disabled') === 'svelte') {
            await manualDownloadGTFS()
        }
    } catch (e) {
        gtfsCache = {header: undefined, entity: [], alerts: []}
    }
    serviceCache = {}
    createAlertCaches()
}

export async function manualDownloadGTFS() {
    const sources = [load_gtfs_source(), load_ember(), load_all_stagecoach_data(), load_passenger_sources(), load_first_vehicles(), load_coaches(), load_Lothian_vehicles()]
    const newEntries: DownloadResponse[] = (await Promise.allSettled(sources)).map(p => {
        if(p.status === 'fulfilled') {
            return p.value
        } else {
            console.error(p.reason)
            return emptyDownloadResponse()
        }
    })

    let nowDate = Date.now() / 1000
    gtfsCache = {
        header: {
            gtfsRealtimeVersion: "2.0",
            incrementality: FeedHeader_Incrementality.FULL_DATASET,
            timestamp: Math.floor(Date.now() / 1000)
        },
        entity: newEntries.flatMap(e => e.entities),
        alerts: newEntries.flatMap(e => e.alerts ?? [])
            .filter(a => a.activePeriod.find(ap => ap.start <= nowDate && ap.end >= nowDate))
    }
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