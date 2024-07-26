import type {ServiceData} from "../../../../api.type";
import {error} from "@sveltejs/kit";
import type {PageServerLoad} from "./$types";
import {env} from "$env/dynamic/private";

export const ssr = false;

export const load: PageServerLoad<ServiceData> = async ({params, fetch}) => {
    if(params.type === "bus") {
        return await fetchData(`${env.API_URL}/api/service?id=` + params.service, fetch)
    } else if(params.type === "train") {
        return await fetchData(`/api/train?id=` + params.service, fetch)
    }

    error(404, "Service type not found.");
}

type FetchType = typeof fetch

async function fetchData(url: string, fetch: FetchType): Promise<ServiceData> {
    let resp = await fetch(url)
    if(!resp.ok) error(resp.status < 500 ? resp.status : 503, await resp.json());
    return await resp.json()
}