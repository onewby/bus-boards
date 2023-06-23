import {initGTFS} from "./routes/api/service/gtfs-cache";
import {db} from "./db";

db.open // Doesn't do anything, just ensures the import isn't removed. DB is imported to ensure it is initialised early.
initGTFS().then(_ => console.log("GTFS downloaded"))