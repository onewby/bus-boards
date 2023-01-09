import type {PageLoad} from "./$types";
import {error} from "@sveltejs/kit";

export const load = (({ url }) => {
    if(!url.searchParams.has("query") || url.searchParams.get("query") === "") throw error(400, "No query searched for.")
    return {};
}) satisfies PageLoad;