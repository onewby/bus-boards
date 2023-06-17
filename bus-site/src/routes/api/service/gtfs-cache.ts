import {DateTime} from "luxon";
import {FeedMessage} from "./gtfs-realtime";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import type {ServiceData} from "../../../api.type";

/*
 * Realtime data
 */

let gtfsCache: FeedMessage = {header: undefined, entity: []}
let lastCacheTime = DateTime.now().minus({minute: 10})
// Caches /api/service outputs for tracking services to prevent latency from recalculating delay (cleared on GTFS update)
let serviceCache: Record<string, ServiceData> = {}

let currentFetch: Promise<FeedMessage> | null = null

// Singleton promise to stop GTFS being downloaded many times at once
async function getGTFS() {
    if(currentFetch) return currentFetch
    let call = _getGTFS()
    currentFetch = call
    let cache = await call
    currentFetch = null
    return cache
}

async function _getGTFS() {
    if(lastCacheTime.diffNow().toMillis() <= -30000) {
        const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
        if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache // Fail nicely - provide previous cache

        const zipReader = new ZipReader(gtfsResp.body)
        let file = (await zipReader.getEntries()).shift()
        if(!file) return gtfsCache

        gtfsCache = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))
        lastCacheTime = DateTime.now()
        serviceCache = {}
    }
    return gtfsCache
}

// Locate trip in GTFS cache
export async function findRealtimeTrip(tripID: string) {
    let gtfs = await getGTFS()
    return gtfs.entity.find(entity => entity.vehicle?.trip?.tripId === tripID)
}

// Return a service in the /api/service cache
export function getRTServiceData(tripID: string): ServiceData {
    return serviceCache[tripID]
}

// Insert a service into the /api/service cache
export function cacheService(tripID: string, data: ServiceData) {
    serviceCache[tripID] = data
}