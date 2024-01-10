import {db} from "../db.js";

export const lineSegmentQuery = db.prepare(
    `SELECT DISTINCT code as stop_id, lat as y, long as x FROM stances
             WHERE code IN (
                 SELECT stop_id FROM stop_times
                    INNER JOIN main.trips t on t.trip_id = stop_times.trip_id WHERE t.route_id=?)`
)