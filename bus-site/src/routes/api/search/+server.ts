import type {RequestHandler} from "./$types";
import {error} from "@sveltejs/kit";
import {db} from "../../../db";

const stmt = db.prepare(
    "SELECT name,parent,qualifier,locality FROM stops_search WHERE stops_search MATCH ? || '*' ORDER BY rank LIMIT 5 OFFSET ?")

export const GET: RequestHandler = ({url}) => {
    const query = url.searchParams.get("query")
    if(query == null || query == "") error(400, "No query provided.");
    const page = Number(url.searchParams.get("page")) ?? 0
    if(isNaN(page)) error(400, "Page is not a number.");
    const offset = page * 5
    return new Response(JSON.stringify(stmt.all([query.split(" ").map(w => `"${w}"`).join(" "), offset])))
}