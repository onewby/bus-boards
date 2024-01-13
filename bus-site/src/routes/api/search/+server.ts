import type {RequestHandler} from "./$types";
import {error} from "@sveltejs/kit";
import {db} from "../../../db";
import { LatLng, LatLngBounds } from "../../../leaflet/geo/index.js";

const queryStmt = db.prepare(
    "SELECT name,parent,qualifier,locality FROM stops_search WHERE stops_search MATCH ? || '*' ORDER BY rank LIMIT 5 OFFSET ?")
const locStmt = db.prepare(
    `SELECT s.name, s.locality_name AS parent, qualifier, s.locality FROM stances
                INNER JOIN main.stops s on s.id = stances.stop INNER JOIN main.localities l on l.code = s.locality
                 WHERE stances.lat >= ? AND stances.long >= ? AND stances.lat <= ? AND stances.long <= ?
                GROUP BY s.id ORDER BY pow(AVG(stances.lat)-?,2)+pow(AVG(stances.long)-?,2) LIMIT 5 OFFSET ?`
)

export const GET: RequestHandler = ({url}) => {
    const page = Number(url.searchParams.get("page")) ?? 0
    if(isNaN(page)) error(400, "Page is not a number.");

    const query = url.searchParams.get("query")
    const lat = Number(url.searchParams.get("lat"))
    const lon = Number(url.searchParams.get("lon"))
    const offset = page * 5
    if(query !== null && query !== "") {
        return new Response(JSON.stringify(queryStmt.all([query.split(" ").map(w => `"${w}"`).join(" "), offset])))
    } else if(!isNaN(lat) && !isNaN(lon)) {
        let bounds: LatLngBounds = new LatLng(lat, lon).toBounds(10000)
        return new Response(JSON.stringify(locStmt.all(
            bounds.getSouthWest().lat, bounds.getSouthWest().lng, bounds.getNorthEast().lat, bounds.getNorthEast().lng,
            lat, lon, offset
        )))
    } else {
        error(400, "Query or location not specified.");
    }
}