import {FeedMessage} from "./gtfs-realtime";
import type {ServiceData} from "../../../api.type";
import {GET as serviceGet} from "./+server";
import {env} from "$env/dynamic/private";
import {downloadGTFS as feederDownloadGTFS, gtfsCache as feederGTFSCache} from "../../../../realtime/feeder.ts";

/*
 * Realtime data
 */

export let gtfsCache: FeedMessage = {header: undefined, entity: []}
// Caches /api/service outputs for tracking services to prevent latency from recalculating delay (cleared on GTFS update)
let serviceCache: Record<string, ServiceData> = {}

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
        if((env.GTFS ?? 'svelte') === 'microservice') {
            const gtfsResp = await fetch(env.GTFS_URL ?? "http://localhost:3948")
            if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache // Fail nicely - provide previous cache
            gtfsCache = await gtfsResp.json()
        } else {
            await feederDownloadGTFS()
            gtfsCache = feederGTFSCache
        }
    } catch (e) {
        gtfsCache = {header: undefined, entity: []}
        console.log(e)
    }
    serviceCache = {}
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
