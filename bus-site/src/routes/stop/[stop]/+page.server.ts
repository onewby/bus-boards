import type {PageServerLoad} from "./$types"
import {error} from "@sveltejs/kit";
import type {StopData} from "../../../api.type";

export const load: PageServerLoad<StopData> = async ({params, fetch, url}) => {
    let date = url.searchParams.get("date")
    let resp = await fetch(`/api/stop?id=${params.stop}${date ? "&date=" + date : ""}`)
    if(!resp.ok) throw error(resp.status < 500 ? resp.status : 503, await resp.json())
    let data: StopData = await resp.json()
    return data
}