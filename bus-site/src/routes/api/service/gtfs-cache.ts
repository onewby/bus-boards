import {FeedMessage} from "./gtfs-realtime";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import type {ServiceData} from "../../../api.type";
import {GET as serviceGet} from "./+server";
import {load_all_stagecoach_data} from "./stagecoach";
import {readFileSync, writeFileSync} from "fs";

/*
 * Realtime data
 */

let gtfsCache: FeedMessage = {header: undefined, entity: []}
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
async function downloadGTFS() {
    try {
        const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
        if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache // Fail nicely - provide previous cache

        const zipReader = new ZipReader(gtfsResp.body)
        let file = (await zipReader.getEntries()).shift()
        if(!file) return gtfsCache

        // @ts-ignore
        gtfsCache = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))

        const stagecoach = await load_all_stagecoach_data()
        gtfsCache.entity.push(...stagecoach)

        serviceCache = {}
    } catch (e) {
        gtfsCache = {header: undefined, entity: []}
        serviceCache = {}
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
