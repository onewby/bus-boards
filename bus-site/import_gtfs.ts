import Database from "better-sqlite3";
import type {Statement} from "better-sqlite3";
import {parse as parsePath} from "node:path";
import {fileURLToPath} from "node:url";
import {readdirSync, readFileSync} from "fs";
import {Open} from "unzipper";
import type {CentralDirectory, File} from "unzipper";
import {parse} from "csv-parse";
import {PromisePool} from "@supercharge/promise-pool";
import {MultiBar, Presets, SingleBar} from "cli-progress";
import hasha from "hasha";
import JSON5 from "json5";

const __file = parsePath(fileURLToPath(import.meta.url))

const db = new Database(__file.dir + "/stops.sqlite");
db.pragma('journal_mode = WAL');
db.pragma('foreign_keys = ON')

let tableCreateScript = readFileSync("gtfs/model.sql", {encoding: "utf-8"})
db.exec(tableCreateScript)

const sources = ["stop_times", "trips", "calendar_dates", "calendar", "routes", "agency"]
let addAgency = db.prepare("REPLACE INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang) VALUES (:agency_id, :agency_name, :agency_url, :agency_timezone, :agency_lang)")
let addRoutes = db.prepare("REPLACE INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type) VALUES (:route_id, :agency_id, :route_short_name, :route_long_name, :route_type)")
let addCalendar = db.prepare("REPLACE INTO calendar (service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date) VALUES (:service_id, :monday, :tuesday, :wednesday, :thursday, :friday, :saturday, :sunday, :start_date, :end_date)")
let addCalendarDates = db.prepare("REPLACE INTO calendar_dates (service_id, date, exception_type) VALUES (:service_id, :date, :exception_type)")
let addTrips = db.prepare("REPLACE INTO trips (route_id, service_id, trip_id, trip_headsign) VALUES (:route_id, :service_id, :trip_id, :trip_headsign)")
let addStopTimes = db.prepare("REPLACE INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, timepoint, stop_headsign) VALUES (:trip_id, :arrival_time, :departure_time, :stop_id, :stop_sequence, :timepoint, NULLIF(:stop_headsign, ''))")
let dropAllSource = db.transaction(() => {
    sources.forEach(table => db.prepare(`DELETE FROM ${table}`).run())
})
let getSignature = db.prepare("SELECT hash FROM file_hashes WHERE source=?")
let setSignature = db.prepare("REPLACE INTO file_hashes (source, hash) VALUES (?, ?)")

const download_urls = {
    "txc": [],
    "gtfs": []
}

async function import_zips() {
    let files = readdirSync("gtfs").filter(f => f.endsWith(".zip"))
    let progressBar = new MultiBar({hideCursor: true, linewrap: true}, {
        format: Presets.shades_classic.format + " | {file}",
        barCompleteChar: Presets.shades_classic.barCompleteChar,
        barIncompleteChar: Presets.shades_classic.barIncompleteChar
    })
    let itemsBar = progressBar.create(files.length, 0, {file: "All sources"})
    itemsBar.start(files.length, 0, {file: "All sources"})
    dropAllSource()
    await PromisePool
        .withConcurrency(4)
        .for(files)
        .handleError(e => console.error(e))
        .process(async f => {
            const path = `gtfs/${f}`
            const source = f.substring(0, f.length - 4)

            let bar = progressBar.create(sources.length + 1, 0, {file: f}, {clearOnComplete: true})
            bar.start(sources.length + 1, 0, {file: f})

            const hash = await hasha.fromFile(path, {algorithm: "sha256"})
            const currentHash = getSignature.get(source)
            if(currentHash === undefined || currentHash['hash'] !== hash) {
                const zip = await Open.file(path)
                try {
                    await import_zip(zip, source, bar)
                    setSignature.run(source, hash)
                } catch (e) {
                    console.log(e)
                    progressBar.log(e instanceof Error ? e.toString() : `An error has occurred processing ${f}.`)
                }
            }

            bar.stop()
            itemsBar.increment()
        })
    itemsBar.stop()
    progressBar.stop()
}

async function import_zip(zip: CentralDirectory, source: string, bar: SingleBar) {
    let files: [string, Statement, Object][] = [
        ["agency.txt", addAgency, {}],
        ["routes.txt", addRoutes, {}],
        ["calendar.txt", addCalendar, {}],
        ["calendar_dates.txt", addCalendarDates, {}],
        ["trips.txt", addTrips, {"trip_headsign": null}],
        ["stop_times.txt", addStopTimes, {}]
    ]
    for(const tuple of files) {
        await import_txt_file(zip, source, tuple[0], tuple[1], tuple[2])
        bar.increment()
    }
    db.exec("UPDATE trips SET max_stop_seq=(SELECT max(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)")
    bar.increment()
}

async function import_txt_file(zip: CentralDirectory, source: string, file_name: string, sql: Statement, defaults: Object = {}) {
    let file = zip.files.find(f => f.path == file_name)
    if(file) {
        await insertSource(file, source, sql, defaults)
    } else throw Error(`Could not find ${file_name} in ${source}`)
}

async function insertSource(file: File, source: string, sql: Statement, defaults: Object = {}) {
    for await (const record of stream_csv(file)) {
        Object.entries(defaults).forEach(([k, v]) => {
            if(!(k in record)) record[k] = v
        })
        try {
            sql.run(record)
        } catch (e) {
            throw Error(`Insertion error: ${file.path} for ${source} using ${sql.source}: ${JSON.stringify(record)}. ${e instanceof Error ? e.toString() : ""}`)
        }
    }
}

function stream_csv(file: File) {
    return file.stream().pipe(parse({encoding: "utf-8", cast: false, cast_date: false, columns: true}))
}

type LocalityCode = string
type StopName = string
type Stance = {
    ATCOCode: string
    Lat: number
    Long: number
    Street: string
    Indicator: string
    Arrival: boolean
}

const stops: Record<LocalityCode, Record<StopName, Stance[]>> = JSON5.parse(readFileSync("../localities.json", {encoding: "utf-8"}))
const arrivalBays: Stance[] = []

Object.entries(stops).forEach(([_, locStops]) => {
    Object.entries(locStops).forEach(([_, stances]) => {
        arrivalBays.push(...stances.filter(stance => stance.Arrival))
    })
})

const arrivalList = arrivalBays.map(bay => `'${bay.ATCOCode}'`).join(",")

const select_all = db.prepare(`
    SELECT arrival.ROWID as arr_id, arrival.arrival_time as arrival_time, departure.ROWID as dep_id FROM stop_times AS departure
      INNER JOIN stances st1 on departure.stop_id = st1.code
      INNER JOIN stop_times arrival on arrival.stop_sequence=departure.stop_sequence-1 AND arrival.stop_id IN (${arrivalList})
      INNER JOIN stances st2 on st2.code = arrival.stop_id
      INNER JOIN trips t on t.trip_id = departure.trip_id
    WHERE st1.stop == st2.stop AND departure.trip_id=arrival.trip_id;
`)

const copy_one = db.prepare(`UPDATE stop_times SET arrival_time=? WHERE rowid=?`)
const del_one = db.prepare(`DELETE FROM stop_times WHERE rowid=?`)

const copy_del_transaction = db.transaction((arr_time, dep_id, arr_id) => {
    copy_one.run(arr_time, dep_id)
    del_one.run(arr_id)
})

function clean_arrivals() {
    console.log("Cleaning arrivals")
    const times = select_all.all()
    const times_length = times.length
    times.forEach((val, i) => {
        console.log(`Fixing ${i + 1}/${times_length}`)
        console.log(val['arrival_time'], val['dep_id'], val['arr_id'])
        if(!val['arrival_time'] || !val['dep_id'] || !val['arr_id']) {
            console.log("Won't fix!")
            return
        }
        copy_del_transaction(val['arrival_time'], val['dep_id'], val['arr_id'])
    })
}

function clean_stops() {
    console.log("Removing stops with no departures")
    db.exec("DELETE FROM stops WHERE stops.id NOT IN (SELECT DISTINCT stances.stop FROM stop_times INNER JOIN stances ON stances.code=stop_id);")
}

await import_zips()
console.log("Imported data successfully.")
clean_arrivals()
clean_stops()
db.close()