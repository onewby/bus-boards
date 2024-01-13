import {createServer} from 'http';
import {FeedHeader_Incrementality, FeedMessage} from "../routes/api/service/gtfs-realtime";
import {Worker} from 'worker_threads';
import {type DownloadResponse, emptyDownloadResponse, type StopAlerts} from "./feeder";

const PORT = 3948

const feeds = ["bods", "coaches", "ember", "first", "lothian", "passenger", "stagecoach"]
const caches: Record<string, DownloadResponse> = Object.fromEntries(feeds.map(feed => [feed, emptyDownloadResponse()]))

const workers = feeds.map(feed => {
    let worker = new Worker(new URL(`./runner.js`, import.meta.url), {
        workerData: {
            path: `./src/realtime/feeder_${feed}.ts`,
            data: "run"
        }
    })
    worker.on('message', async (msg: DownloadResponse) => {
        caches[feed] = msg
    })
    return worker
})

createServer((req, res) => {
    let nowDate = Date.now() / 1000
    let cacheValues = Object.values(caches)
    let msg: FeedMessage & StopAlerts = {
        entity: cacheValues.flatMap(e => e.entities),
        header: {
            gtfsRealtimeVersion: "2.0",
            incrementality: FeedHeader_Incrementality.FULL_DATASET,
            timestamp: Math.floor(Date.now() / 1000)
        },
        alerts: cacheValues.flatMap(e => e.alerts ?? [])
            .filter(a => a.activePeriod.find(ap => ap.start <= nowDate && ap.end >= nowDate))
    }
    res.writeHead(200, {'Content-Type': 'application/json'})
    res.write(JSON.stringify(msg))
    res.end()
}).listen(PORT)