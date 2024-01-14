import {
    type Alert, Alert_Cause, Alert_Effect, type EntitySelector,
    FeedMessage, type TimeRange
} from "../routes/api/service/gtfs-realtime.js";
import {Uint8ArrayWriter, ZipReader} from "@zip.js/zip.js";
import {type DownloadResponse, Feeder, emptyDownloadResponse} from "./feeder.js";
import {XMLParser} from "fast-xml-parser"
import type {SiriSx} from "../api.type.ts";
import {DateTime} from "luxon";
import {db} from "../db.js";

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
        alerts: await load_disruptions()
    }
}

const getAgency = db.prepare("SELECT agency_id FROM traveline WHERE code=?").pluck()
const getRoute = db.prepare("SELECT route_id FROM routes INNER JOIN main.traveline t on routes.agency_id = t.agency_id WHERE code=? AND route_short_name=?").pluck()

export async function load_disruptions(): Promise<Alert[]> {
    const siriResp = await fetch("https://data.bus-data.dft.gov.uk/api/v1/siri-sx/")
    if(!siriResp.ok) return []
    const disruptions: SiriSx = new XMLParser().parse(await siriResp.text())
    return disruptions.Siri.ServiceDelivery.SituationExchangeDelivery.Situations.PtSituationElement.flatMap(situation => {
        let timeRanges: TimeRange[] = compact(situation.ValidityPeriod).map(vp => ({
            start: vp.StartTime ? DateTime.fromISO(vp.StartTime).toSeconds() : 0,
            end: vp.EndTime ? DateTime.fromISO(vp.EndTime).toSeconds() : DateTime.now().plus({year: 1}).toSeconds()
        }))
        // One generic for stops
        // Specific advice for each route
        let consequences = compact(situation.Consequences.Consequence)
        let alerts: Alert[] = consequences.map(con => {
            return {
                activePeriod: timeRanges,
                cause: Alert_Cause.OTHER_CAUSE,
                effect: Alert_Effect.OTHER_EFFECT,
                descriptionText: {translation: [{text: situation.Description + (con.Advice?.Details ? ` ${con.Advice.Details}` : ''), language: "en"}]},
                headerText: {translation: [{text: situation.Summary, language: "en"}]},
                informedEntity: [
                    // AllLines ignored due to irrelevant data showing
                    ...compact(con.Affects.Networks.AffectedNetwork.AffectedLine).map(line => {
                        let opCode = line.AffectedOperator.OperatorRef
                        let routeName = line.LineRef
                        let route = getRoute.get(opCode, routeName)
                        if(route === undefined) return undefined
                        return {
                            agencyId: "", routeId: route, routeType: 0, trip: undefined, stopId: ""
                        };
                    }).filter(obj => obj !== undefined),
                    ...compact(con.Affects.Operators?.AffectedOperators).map(op => {
                        let agency = getAgency.get(op.OperatorRef)
                        if(agency === undefined) return undefined
                        return {
                            agencyId: agency, routeId: "", routeType: 0, trip: undefined, stopId: ""
                        };
                    }).filter(obj => obj !== undefined)
                ] as EntitySelector[],
                url: situation.InfoLinks?.InfoLink.Uri ? {translation: [{text: situation.InfoLinks?.InfoLink.Uri, language: "en"}]} : undefined
            }
        })
        let stopPoints = consequences.flatMap(con => compact(con.Affects.StopPoints?.AffectedStopPoint))
        if(stopPoints.length > 0) {
            alerts.push({
                activePeriod: timeRanges,
                cause: Alert_Cause.OTHER_CAUSE,
                effect: Alert_Effect.OTHER_EFFECT,
                descriptionText: {translation: [{text: situation.Description, language: "en"}]},
                headerText: {translation: [{text: situation.Summary, language: "en"}]},
                informedEntity: stopPoints.map(sp => {
                    return {
                        agencyId: "", routeId: "", routeType: 0, trip: undefined,
                        stopId: String(sp.StopPointRef)
                    };
                }),
                url: situation.InfoLinks?.InfoLink.Uri ? {translation: [{text: situation.InfoLinks?.InfoLink.Uri, language: "en"}]} : undefined
            })
        }
        return alerts
    })
}

function compact<T>(obj: T | T[] | undefined): T[] {
    return obj === undefined ? [] : Array.isArray(obj) ? obj : [obj]
}

new Feeder(load_gtfs_source).init()