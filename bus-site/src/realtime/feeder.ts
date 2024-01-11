import {parentPort} from "worker_threads";
import type {Alert, FeedEntity} from "../routes/api/service/gtfs-realtime.js";
import {workerData} from "node:worker_threads";

export type StopAlerts = {
    alerts: Record<string, Alert[]>
}
export type DownloadResponse = {
    entities: FeedEntity[]
    stopAlerts: Record<string, Alert[]>
}
type DownloadFunction = () => Promise<DownloadResponse>

export const emptyDownloadResponse = () => ({entities: [], stopAlerts: {}})

export class Feeder {

    downloadFunction: DownloadFunction;
    constructor(downloadFunction: DownloadFunction) {
        this.downloadFunction = downloadFunction
    }

    init() {
        if(this.isMainFile()) this.run(true);
    }

    isMainFile() {
        return workerData === "run"
    }

    run(initial = false) {
        setTimeout(async () => {
            if(parentPort !== null) {
                let feed: DownloadResponse = emptyDownloadResponse()
                try {
                    feed = await this.downloadFunction()
                } catch (e) {}
                parentPort.postMessage(feed)
            }
            this.run()
        }, initial ? 0 : 10000)
    }
}