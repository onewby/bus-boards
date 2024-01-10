import {createServer} from 'http';
import {FeedEntity, FeedHeader_Incrementality, FeedMessage} from "../routes/api/service/gtfs-realtime.js";
import { Worker } from 'worker_threads';

const PORT = 3948

const feeds = ["bods", "coaches", "ember", "first", "passenger", "stagecoach"]
const caches: Record<string, FeedEntity[]> = Object.fromEntries(feeds.map(feed => [feed, []]))

const workers = feeds.map(feed => {
    let worker = new Worker(new URL(`./feeder_${feed}.js`, import.meta.url), {workerData: "run"})
    worker.on('message', (msg: FeedEntity[]) => {
        caches[feed] = msg
    })
    return worker
})

createServer((req, res) => {
    let msg: FeedMessage = {
        entity: Object.values(caches).flat(),
        header: {
            gtfsRealtimeVersion: "2.0",
            incrementality: FeedHeader_Incrementality.FULL_DATASET,
            timestamp: Math.floor(Date.now() / 1000)
        }
    }
    res.writeHead(200, {'Content-Type': 'application/json'})
    res.write(JSON.stringify(msg))
    res.end()
}).listen(PORT)