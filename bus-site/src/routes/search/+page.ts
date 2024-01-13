import type {PageLoad} from "./$types";
import {error} from "@sveltejs/kit";

export const load = (({ url }) => {
    if((!url.searchParams.has("query") || url.searchParams.get("query") === "")
        && (!url.searchParams.has("lat") || url.searchParams.get("lat") === ""
            || !url.searchParams.has("lon") || url.searchParams.get("lon") === ""))
        error(400, "No query searched for.");
    if(isNaN(Number(url.searchParams.get("lat"))) && isNaN(Number(url.searchParams.get("lon"))))
        error(400, "Invalid lat/lon parameters.")
    return {};
}) satisfies PageLoad;