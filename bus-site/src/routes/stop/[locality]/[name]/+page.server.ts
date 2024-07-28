import type {PageServerLoad} from "./$types"
import {error} from "@sveltejs/kit";
import type {SearchResult, StopData} from "../../../../api.type";
import {API_URL} from "$env/static/private";

export const ssr = false;

export const load: PageServerLoad<StopData> = async ({params, fetch, url}) => {
    let date = url.searchParams.get("date")
    let filterLoc = url.searchParams.get("filterLoc")
    let filterName = url.searchParams.get("filterName")

    let apiParams = new URLSearchParams()
    apiParams.set("locality", params.locality)
    apiParams.set("name", params.name)
    if(date) apiParams.set("date", date)
    if(filterLoc && filterName) {
        apiParams.set("filterLoc", filterLoc)
        apiParams.set("filterName", filterName)
    }

    let resp = await fetch(`${API_URL}/api/stop?${apiParams}`)
    if(!resp.ok) error(resp.status < 500 ? resp.status : 503, await resp.json());
    let data: StopData = await resp.json()

    if(filterLoc && filterName) {
        let apiParams = new URLSearchParams()
        apiParams.set("locality", filterLoc)
        apiParams.set("name", filterName)
        let filterResp = await fetch(`${API_URL}/api/stop?${apiParams}`)
        if(filterResp.ok) {
            let filterData: StopData = await filterResp.json()
            data.filter = {
                name: filterData.stop.name,
                parent: filterData.stop.locality_name,
                locality: filterData.stop.locality_code,
                qualifier: filterData.stop.locality_name
            }
        }
    }

    return data
}