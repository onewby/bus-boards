import {type Alert, FeedEntity, FeedMessage} from "../routes/api/service/gtfs-realtime.js";
import {type DownloadResponse, Feeder, emptyDownloadResponse} from "./feeder.js";

export async function load_ember(): Promise<DownloadResponse> {
    const resp = await fetch("https://api.ember.to/v1/gtfs/realtime/")
    if(!resp.ok || !resp.body) return emptyDownloadResponse()
    let entities = FeedMessage.decode(new Uint8Array(await resp.arrayBuffer())).entity
    let stopAlerts: Record<string, Alert[]> = {}
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
            if(entity.trip) {
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
            } else if(entity.stopId) {
                if(!stopAlerts[entity.stopId]) {
                    stopAlerts[entity.stopId] = [vehicleless.alert!]
                } else {
                    stopAlerts[entity.stopId].push(vehicleless.alert!)
                }
            }
        })
    })
    return {
        entities: entities.filter(trip => trip.vehicle), stopAlerts
    }
}

new Feeder(load_ember).init()