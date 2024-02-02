import {parentPort} from "worker_threads";
import type {Alert, FeedEntity} from "../routes/api/service/gtfs-realtime";
import {workerData} from "node:worker_threads";
import {existsSync} from "node:fs";
import {DateTime} from "luxon";
import {readFileSync, writeFileSync} from "fs";

export type StopAlerts = {
    alerts: Alert[]
}
export type DownloadResponse = {
    entities: FeedEntity[]
    alerts?: Alert[]
}
type DownloadFunction = () => Promise<DownloadResponse>

export const emptyDownloadResponse = () => ({entities: []})

export class Feeder {

    downloadFunction: DownloadFunction;
    constructor(downloadFunction: DownloadFunction) {
        this.downloadFunction = downloadFunction
    }

    init() {
        if(this.isMainFile()) this.run(true);
    }

    isMainFile() {
        return workerData?.data === "run"
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

export class UpdateFeeder extends Feeder {

    lastUpdate = existsSync(".update") ? DateTime.fromISO(readFileSync(".update", "utf-8")) : DateTime.now().minus({days: 5, hours: 1})
    updateFunction: Function

    async checkUpdate() {
        if (this.lastUpdate.diffNow("days").days <= -5) {
            this.lastUpdate = DateTime.now().set({hour: 2, minute: 0, second: 0, millisecond: 0})
            await this.updateFunction()
            writeFileSync(".update", this.lastUpdate.toISO()!)
        }
    }

    constructor(downloadFunction: DownloadFunction, updateFunction: Function) {
        super(async () => {
            await this.checkUpdate()
            return downloadFunction()
        });
        this.updateFunction = updateFunction
    }
}