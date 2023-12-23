import {parentPort} from "worker_threads";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import {FeedMessage} from "../src/routes/api/service/gtfs-realtime.js";
import {load_all_stagecoach_data} from "../src/routes/api/service/stagecoach.js";
import {load_passenger_sources} from "../src/routes/api/service/passenger.js";

let gtfsCache: FeedMessage

export async function initGTFS() {
    await downloadGTFS()
    publish()
    gtfsUpdateLoop()
}

function gtfsUpdateLoop() {
    setTimeout(async () => {
        await downloadGTFS()
        publish()
        gtfsUpdateLoop()
    }, 10000)
}

export async function downloadGTFS() {
    try {
        const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
        if(!gtfsResp.ok || !gtfsResp.body) return gtfsCache // Fail nicely - provide previous cache

        const zipReader = new ZipReader(gtfsResp.body)
        let file = (await zipReader.getEntries()).shift()
        if(!file) return gtfsCache

        // @ts-ignore
        const newCache = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))

        const sources = [load_all_stagecoach_data(), load_passenger_sources()]
        newCache.entity.push(...(await Promise.all(sources)).flat())

        gtfsCache = newCache
    } catch (e) {
        gtfsCache = {header: undefined, entity: []}
        console.log(e)
    }
}

function publish() {
    parentPort!.postMessage(gtfsCache)
}

initGTFS()