import type {RequestHandler} from "./$types";
import {error, json} from "@sveltejs/kit";
import {darwin} from "../darwin";
import type {CallingPoint, ServiceDetails} from "../../../darwin/darwin";
import type {ServiceBranch, ServiceStopData} from "../../../api.type";
import {DateTime} from "luxon";
import {db} from "../../../db";
import polyline from "google-polyline";

const UK_CTR_LONG = -2.547855
const UK_CTR_LAT = 54.00366

export const GET: RequestHandler = async ({url}) => {
    let id = url.searchParams.get("id")
    if(id === null) error(404, "ID not provided.");

    let details: ServiceDetails
    try {
        details = await darwin.getServiceDetails(id)
    } catch (e) {
        error(404, "Cannot find service.");
    }

    const prevCalls = details.previousCallingPoints.callingPointList
    const subsequentCalls = details.subsequentCallingPoints.callingPointList

    const service = {
        code: details.std ?? details.sta,
        dest: subsequentCalls.map(list => list.callingPoint[list.callingPoint.length - 1].locationName).join(' & '),
        cancelled: details.isCancelled || details.etd === "Cancelled"
    }
    const alerts = details.cancelReason || details.delayReason ? [{
        header: undefined,
        description: details.isCancelled ? (details.cancelReason ?? details.delayReason) : (details.delayReason ?? details.cancelReason),
        url: undefined
    }] : []
    const operator = {
        name: details.operator,
        url: "https://www.nationalrail.co.uk/"
    }

    const branches: ServiceBranch[] = []
    let numBranches = Math.max(prevCalls.length, subsequentCalls.length)
    for(let i = 0; i < numBranches; i++) {
        const cps = (prevCalls[Math.min(i, prevCalls.length - 1)]?.callingPoint ?? [])
            .concat({
                    locationName: details.locationName,
                    crs: details.crs,
                    st: details.std ?? details.sta!,
                    et: details.etd ?? details.eta,
                    at: details.atd ?? details.ata},
                ...(subsequentCalls[Math.min(i, subsequentCalls.length - 1)]?.callingPoint ?? []))

        const dest = cps[cps.length - 1].locationName
        const params = cps.map(cp => `'${cp.crs}'`).join(",")
        const coordsQuery = db.prepare(
            `SELECT crs,lat,long,s.name as name,s.locality as locality FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE crs IN (${params})`
        ).all() as CRSResponse[]
        const coords: Record<string, Record<string, any>> = {}
        coordsQuery.forEach(result => coords[result['crs']] = result)

        const stops: ServiceStopData[] =
            cps.map((cp, i) => ({
                name: coords[cp.crs]?.['name'] ?? "",
                display_name: cp.locationName,
                loc: undefined,
                ind: cp.crs === details.crs && details.platform ? "Platform " + details.platform : undefined,
                arr: cp.st,
                dep: cp.st,
                puo: false,
                doo: false,
                major: true,
                long: coords[cp.crs]?.['long'] ?? UK_CTR_LONG,
                lat: coords[cp.crs]?.['lat'] ?? UK_CTR_LAT,
                status: cp.et ? (isNum(cp.et[0]) ? "Exp. " + cp.et : cp.et)
                    : cp.at ? (isNum(cp.at[0]) ? "Dep. " + cp.at : cp.at) : undefined,
                seq: i,
                locality: coords[cp.crs]?.['locality'] ?? ""
            }))
        let currStop = cps.findLastIndex(stop => stop.at != undefined)
        let nextStop = currStop + 1
        let currTime = getTime(cps[currStop] ?? cps[0])

        const realtime = {
            stop: nextStop,
            pos: undefined,
            pct: currStop == -1 ? undefined : nextStop === cps.length ? 1
                : Math.min(Math.abs(currTime.diffNow().milliseconds / (currTime.diff(getTime(cps[nextStop]))).milliseconds), 1)
        }
        const route: string = polyline.encode(cps.filter(cp => coords[cp.crs]?.['lat']).map(cp => [coords[cp.crs]?.["lat"] ?? UK_CTR_LAT, coords[cp.crs]?.["long"] ?? UK_CTR_LONG]))
        branches.push({
            "dest": dest,
            "stops": stops,
            "realtime": realtime,
            "route": route
        })
    }

    return json({
        "service": service,
        "operator": operator,
        "branches": branches,
        "alerts": alerts
    })
}

function getTime(point: CallingPoint): DateTime {
    let time = point.at != undefined && isNum(point.at[0]) ? point.at : (point.et != undefined && isNum(point.et[0]) ? point.et : point.st)
    return DateTime.fromFormat(time, "HH:mm")
}

const isNum = (c: string) => c >= '0' && c <= '9'

type CRSResponse = {
    crs: string,
    lat: number,
    long: number,
    name: string,
    locality: string
}