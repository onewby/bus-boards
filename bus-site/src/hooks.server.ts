import {initGTFS} from "./routes/api/service/gtfs-cache";

initGTFS().then(_ => console.log("GTFS downloaded"))