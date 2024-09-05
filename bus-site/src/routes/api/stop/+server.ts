import type {RequestHandler} from "./$types";
import {API_URL} from "$env/static/private";
import {error, json} from "@sveltejs/kit";

export const GET: RequestHandler = async ({url}) => {
    let params = url.searchParams.toString()
    let resp = await fetch(`${API_URL}/api/stop?${params}`)
    if(resp.ok) {
        return json(await resp.json())
    } else {
        error(resp.status, {message: await resp.text()})
    }
}