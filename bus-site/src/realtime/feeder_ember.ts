import {FeedEntity, FeedMessage} from "../routes/api/service/gtfs-realtime.js";
import {Feeder} from "./feeder.js";

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

new Feeder(load_ember).init()