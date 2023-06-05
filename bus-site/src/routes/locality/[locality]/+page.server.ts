import type {PageServerLoad} from "./$types";
import {db} from "../../../db";
import {error} from "@sveltejs/kit";

const dbInfo = db.prepare("SELECT name, parent FROM localities WHERE code=?");
const dbStops = db.prepare("SELECT stop.id, stop.name, avg(lat) AS lat, avg(long) AS long FROM stances INNER JOIN stops stop on stances.stop = stop.id WHERE stop.locality=? GROUP BY stop ORDER BY name;")
const dbParent = db.prepare(`
    SELECT GROUP_CONCAT(name, ' › ') AS parent FROM (
        WITH RECURSIVE
            find_parent_names(level, code) AS (
                VALUES(0, ?)
                UNION
                SELECT level+1, parent FROM localities, find_parent_names
                WHERE localities.code=find_parent_names.code
            )
        SELECT name FROM localities, find_parent_names
        WHERE localities.code = find_parent_names.code
        ORDER BY level desc
    )
`)
const dbSublocalities = db.prepare("SELECT code as id, name, lat, long FROM localities WHERE parent=? ORDER BY name")

export const load: PageServerLoad = async ({params}) => {
    if(!params.locality) throw error(400, "Locality code not specified")
    let info = dbInfo.get(params.locality)
    if(!info) throw error(404)
    let parent = info.parent ? dbParent.get(info.parent).parent : undefined
    let stops: {id: string, name: string, lat: number, long: number}[] = dbStops.all(params.locality)
    let sublocs: {id: string, name: string, lat: number, long: number}[] = dbSublocalities.all(params.locality)

    return {
        "name": info.name,
        "parent": {
            "id": info.parent,
            "name": parent
        },
        "children": sublocs,
        "results": stops
    }
}