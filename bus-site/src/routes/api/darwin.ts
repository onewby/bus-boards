// National Rail API
import {DarwinAPI} from "../../darwin/darwin";
import {env} from "$env/dynamic/private";

export const darwin = new DarwinAPI(env.DARWIN_API_KEY ?? "")