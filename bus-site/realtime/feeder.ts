import {parentPort} from "worker_threads";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import {FeedMessage} from "../src/routes/api/service/gtfs-realtime.js";
import {load_all_stagecoach_data} from "../src/routes/api/service/stagecoach.js";
import {load_passenger_sources} from "../src/routes/api/service/passenger.js";
import {downloadRouteDirections} from "../import_passenger.js";
import {DateTime} from "luxon";
import {workerData} from "node:worker_threads";
import {load_first_vehicles} from "../src/routes/api/service/first.js";
import {existsSync} from "node:fs";
import {readFileSync, writeFileSync} from "fs";

export let gtfsCache: FeedMessage
let lastUpdate = existsSync(".update") ? DateTime.fromISO(readFileSync(".update", "utf-8")) : DateTime.fromSQL("02:00:00")

export async function initGTFS() {
    await checkPassengerUpdate()
    await downloadGTFS()
    publish()
    gtfsUpdateLoop()
}

function gtfsUpdateLoop() {
    setTimeout(async () => {
        await downloadGTFS()
        publish()
        await checkPassengerUpdate()
        gtfsUpdateLoop()
    }, 10000)
}

async function checkPassengerUpdate() {
    if(lastUpdate.diffNow("days").days <= -5) {
        await downloadRouteDirections()
        lastUpdate = DateTime.now()
        writeFileSync(".update", DateTime.now().toISO()!)
    }
}

export async function downloadGTFS() {
    try {
        const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
        if(!gtfsResp.ok || !gtfsResp.body) return // Fail nicely - provide previous cache

        const zipReader = new ZipReader(gtfsResp.body)
        let file = (await zipReader.getEntries()).shift()
        if(!file) return

        // @ts-ignore
        const newCache = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))
        newCache.entity = newCache.entity.filter(e => e.vehicle?.trip?.tripId !== "")

        const sources = [load_all_stagecoach_data(), load_passenger_sources(), load_first_vehicles()]
        newCache.entity.push(...(await Promise.all(sources)).flat())

        gtfsCache = newCache
    } catch (e) {
        gtfsCache = {header: undefined, entity: []}
        console.log(e)
    }
}

function publish() {
    if(parentPort !== null) parentPort.postMessage(gtfsCache)
}

if(workerData === "run") initGTFS()