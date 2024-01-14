import {FeedMessage} from "../routes/api/service/gtfs-realtime";
import {type DownloadResponse, Feeder, emptyDownloadResponse} from "./feeder";

export async function load_ember(): Promise<DownloadResponse> {
    const resp = await fetch("https://api.ember.to/v1/gtfs/realtime/")
    if(!resp.ok || !resp.body) return emptyDownloadResponse()
    let entities = FeedMessage.decode(new Uint8Array(await resp.arrayBuffer())).entity
    // Ember feed contains separate vehicle positioning and delay update feed entities - we can merge them
    entities.filter(trip => trip.tripUpdate && !trip.vehicle).forEach(vehicleless => {
        let trip = entities.find(tripless =>
            !tripless.tripUpdate && tripless.vehicle?.trip?.tripId === vehicleless.tripUpdate?.trip?.tripId)
        if(trip) trip.tripUpdate = vehicleless.tripUpdate
    })
    let alerts = entities.filter(trip => trip.alert).map(trip => trip.alert!)
    return {
        entities: entities.filter(trip => trip.vehicle), alerts
    }
}

new Feeder(load_ember).init()