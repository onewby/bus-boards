import {FeedEntity, FeedMessage} from "../routes/api/service/gtfs-realtime.js";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import {type DownloadResponse, Feeder, emptyDownloadResponse} from "./feeder.js";

export async function load_gtfs_source(): Promise<DownloadResponse> {
    const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
    if(!gtfsResp.ok || !gtfsResp.body) return emptyDownloadResponse() // Fail nicely - provide previous cache

    const zipReader = new ZipReader(gtfsResp.body)
    let file = (await zipReader.getEntries()).shift()
    if(!file) return emptyDownloadResponse()

    // @ts-ignore
    const entries = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))
    entries.entity = entries.entity.filter(e => e.vehicle?.trip?.tripId !== "")

    return {
        entities: entries.entity,
        stopAlerts: {}
    }
}

new Feeder(load_gtfs_source).init()