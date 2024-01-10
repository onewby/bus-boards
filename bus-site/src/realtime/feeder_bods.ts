import {FeedEntity, FeedMessage} from "../routes/api/service/gtfs-realtime.js";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import {Feeder} from "./feeder.js";

export async function load_gtfs_source(): Promise<FeedEntity[]> {
    const gtfsResp = await fetch("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt")
    if(!gtfsResp.ok || !gtfsResp.body) return [] // Fail nicely - provide previous cache

    const zipReader = new ZipReader(gtfsResp.body)
    let file = (await zipReader.getEntries()).shift()
    if(!file) return []

    // @ts-ignore
    const entries = FeedMessage.decode(await file.getData(new Uint8ArrayWriter()))
    entries.entity = entries.entity.filter(e => e.vehicle?.trip?.tripId !== "")

    return entries.entity
}

new Feeder(load_gtfs_source).init()