import {parentPort} from "worker_threads";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import {type FeedEntity, FeedHeader_Incrementality, FeedMessage} from "../src/routes/api/service/gtfs-realtime.js";
import {load_all_stagecoach_data} from "../src/routes/api/service/stagecoach.js";
import {load_passenger_sources} from "../src/routes/api/service/passenger.js";
import {downloadRouteDirections} from "../import_passenger.js";
import {DateTime} from "luxon";
import {workerData} from "node:worker_threads";
import {initialise_first, load_first_vehicles} from "../src/routes/api/service/first.js";
import {existsSync} from "node:fs";
import {readFileSync, writeFileSync} from "fs";
import {load_coaches} from "../src/routes/api/service/coaches.js";

export let gtfsCache: FeedMessage
let lastUpdate = existsSync(".update") ? DateTime.fromISO(readFileSync(".update", "utf-8")) : DateTime.now().minus({days: 5, hours: 1})

export async function initGTFS() {
    await checkPassengerUpdate()
    await initialise_first()
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
        lastUpdate = DateTime.now().set({hour: 2, minute: 0, second: 0, millisecond: 0})
        writeFileSync(".update", DateTime.now().toISO()!)
    }
}

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

export async function load_ember(): Promise<FeedEntity[]> {
    const resp = await fetch("https://api.ember.to/v1/gtfs/realtime/")
    if(!resp.ok || !resp.body) return []
    let entities = FeedMessage.decode(new Uint8Array(await resp.arrayBuffer())).entity
    // Ember feed contains separate vehicle positioning and delay update feed entities - we can merge them
    entities.filter(trip => trip.tripUpdate && !trip.vehicle).forEach(vehicleless => {
        let trip = entities.find(tripless => !tripless.tripUpdate && tripless.vehicle?.trip?.tripId === vehicleless.tripUpdate?.trip?.tripId)
        if(trip) trip.tripUpdate = vehicleless.tripUpdate
    })
    // Integrate trip-based alerts: merge descriptions if multiple
    entities.filter(trip => trip.alert && !trip.vehicle).forEach(vehicleless => {
        let alertText = vehicleless.alert!.descriptionText?.translation[0].text
        if(!alertText) return
        vehicleless.alert!.informedEntity.forEach(entity => {
            let trip = entities.find(unalerted => entity.trip?.tripId && unalerted.vehicle?.trip?.tripId === entity.trip?.tripId)
            if(trip) {
                if(trip.alert) {
                    let text = trip.alert.descriptionText?.translation[0].text ?? ""
                    if(!text.includes(alertText!)) {
                        if(!trip.alert.descriptionText) trip.alert.descriptionText = {translation: [{text: "", language: "en"}]}
                        trip.alert.descriptionText.translation[0].text = text + '\n' + alertText
                    }
                } else {
                    trip.alert = structuredClone(vehicleless.alert)
                }
            }
        })
    })
    return entities.filter(trip => trip.vehicle)
}

export async function downloadGTFS() {
    const sources = [load_gtfs_source(), load_ember(), load_all_stagecoach_data(), load_passenger_sources(), load_first_vehicles(), load_coaches()]
    const newEntries = (await Promise.allSettled(sources)).map(p => {
        if(p.status === 'fulfilled') {
            return p.value
        } else {
            console.error(p.reason)
            return []
        }
    }).flat()

    gtfsCache = {
        header: {
            gtfsRealtimeVersion: "2.0",
            incrementality: FeedHeader_Incrementality.FULL_DATASET,
            timestamp: Math.floor(Date.now() / 1000)
        }, entity: newEntries
    }
}

function publish() {
    if(parentPort !== null) parentPort.postMessage(gtfsCache)
}

if(workerData === "run") initGTFS()