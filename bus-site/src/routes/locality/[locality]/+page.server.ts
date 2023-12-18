import type {PageServerLoad} from "./$types";
import {db} from "../../../db";
import {error} from "@sveltejs/kit";

type LInfo = {name: string, parent: string}
type LStop = {id: string, locality: string, name: string, lat: number, long: number}
type LSubloc = {id: string, name: string, lat: number, long: number}
type LParent = {name: string, parent: string}
const dbInfo = db.prepare("SELECT name, parent FROM localities WHERE code=?");
const dbStops = db.prepare("SELECT stop.id, stop.locality, stop.name, avg(lat) AS lat, avg(long) AS long FROM stances INNER JOIN stops stop on stances.stop = stop.id WHERE stop.locality=? GROUP BY stop ORDER BY name;")
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
    if(!params.locality) error(400, "Locality code not specified");
    let info: LInfo = dbInfo.get(params.locality) as LInfo
    if(!info) error(404);
    let parent: string | undefined = info.parent ? (dbParent.get(info.parent) as LParent).parent : undefined
    let stops: LStop[] = dbStops.all(params.locality) as LStop[]
    let sublocs: LSubloc[] = dbSublocalities.all(params.locality) as LSubloc[]

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