import {createServer} from 'http';
import {FeedMessage} from "../src/routes/api/service/gtfs-realtime.js";
import { Worker } from 'worker_threads';

const PORT = 3948

const worker = new Worker(new URL('./feeder.js', import.meta.url));

let gtfsCache = JSON.stringify({header: undefined, entity: []})

worker.on('message', (msg: FeedMessage) => {
    gtfsCache = JSON.stringify(msg)
})

createServer((req, res) => {
    res.write(gtfsCache)
    res.end()
}).listen(PORT)