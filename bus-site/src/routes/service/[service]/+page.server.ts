import type {ServiceData} from "../../../api.type";
import {error} from "@sveltejs/kit";
import type {PageServerLoad} from "./$types";

export const load: PageServerLoad<ServiceData> = async ({params, fetch}) => {
    let resp = await fetch(`/api/service?id=` + params.service)
    if(!resp.ok) throw error(resp.status < 500 ? resp.status : 503, await resp.json())
    let data: ServiceData = await resp.json()
    return data
}