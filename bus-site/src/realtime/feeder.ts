import {parentPort} from "worker_threads";
import type {FeedEntity} from "../routes/api/service/gtfs-realtime.js";
import {workerData} from "node:worker_threads";

type DownloadFunction = () => Promise<FeedEntity[]>

export class Feeder {

    downloadFunction: DownloadFunction;
    constructor(downloadFunction: DownloadFunction) {
        this.downloadFunction = downloadFunction
    }

    init() {
        if(this.isMainFile()) this.run();
    }

    isMainFile() {
        return workerData === "run"
    }

    run() {
        setTimeout(async () => {
            if(parentPort !== null) {
                let feed: FeedEntity[] = []
                try {
                    feed = await this.downloadFunction()
                } catch (e) {}
                parentPort.postMessage(feed)
            }
            this.run()
        }, 10000)
    }
}