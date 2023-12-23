import Database from "better-sqlite3";
import type {Statement} from "better-sqlite3";
import {parse as parsePath} from "node:path";
import {fileURLToPath} from "node:url";
import {readFileSync, writeFileSync, createWriteStream} from "fs";
import { Readable } from 'stream';
import { finished } from 'stream/promises';
import {Open} from "unzipper";
import type {CentralDirectory, File} from "unzipper";
import {parse} from "csv-parse";
import hasha from "hasha";
import JSON5 from "json5";
import {downloadRouteDirections} from "./import_passenger";

const __file = parsePath(fileURLToPath(import.meta.url))

const db = new Database(__file.dir + "/stops.sqlite");
db.pragma('journal_mode = WAL');
db.pragma('foreign_keys = ON')

let indexingScript = readFileSync("gtfs/indexes.sql", {encoding: "utf-8"})
let tableCreateScript = readFileSync("gtfs/model.sql", {encoding: "utf-8"})
db.exec(tableCreateScript)

const sources = ["stop_times", "trips", "calendar_dates", "calendar", "routes", "agency"]
let addAgency = db.prepare("REPLACE INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang) VALUES (:agency_id, :agency_name, :agency_url, :agency_timezone, :agency_lang)")
let addRoutes = db.prepare("REPLACE INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type) VALUES (:route_id, :agency_id, :route_short_name, :route_long_name, :route_type)")
let addCalendar = db.prepare("REPLACE INTO calendar (service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date) VALUES (:service_id, :monday, :tuesday, :wednesday, :thursday, :friday, :saturday, :sunday, :start_date, :end_date)")
let addCalendarDates = db.prepare("REPLACE INTO calendar_dates (service_id, date, exception_type) VALUES (:service_id, :date, :exception_type)")
let addTrips = db.prepare("REPLACE INTO trips (route_id, service_id, trip_id, trip_headsign) VALUES (:route_id, :service_id, :trip_id, :trip_headsign)")
let addStopTimes = db.prepare("REPLACE INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, timepoint, stop_headsign, pickup_type, drop_off_type) VALUES (:trip_id, :arrival_time, :departure_time, :stop_id, :stop_sequence, :timepoint, NULLIF(:stop_headsign, ''), :pickup_type, :drop_off_type)")
let dropAllSource = db.transaction(() => {
    db.pragma('foreign_keys = OFF')
    sources.forEach(table => db.prepare(`DELETE FROM ${table}`).run())
    db.exec(tableCreateScript)
    db.pragma('foreign_keys = ON')
})

async function import_zips() {
    const path = "gtfs/itm_all_gtfs.zip"

    if(process.argv.length < 3 || process.argv[2].toLowerCase() != "-s") {
        console.log("Downloading GTFS data (usually ~550MB)")
        let resp = await fetch("https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/")
        if(!resp.ok || !resp.body) {
            console.error("Cannot download GTFS data")
            return
        }
        let fileStream = createWriteStream(path)
        // @ts-ignore
        await finished(Readable.fromWeb(resp.body).pipe(fileStream))
        fileStream.close()

        if(process.argv.length < 3 || process.argv[2].toLowerCase() != "-h") {
            const hash = await hasha.fromFile(path, {algorithm: "sha256"})
            let currentHash = undefined
            try {
                currentHash = readFileSync("gtfs/hash.txt", "utf-8")
            } catch (e) {}

            if(currentHash !== undefined && currentHash === hash) {
                console.log("GTFS up to date - run ts-node-esm import_gtfs.ts -h to ignore hash")
            }
        }
    }

    console.log("Dropping previous data")
    dropAllSource()
    console.log("Inserting new data")

    const zip = await Open.file(path)
    try {
        await import_zip(zip)

        const hash = await hasha.fromFile(path, {algorithm: "sha256"})
        writeFileSync("gtfs/hash.txt", hash)
    } catch (e) {
        console.log(e)
    }

    console.log("Imported data successfully.")
}

async function import_zip(zip: CentralDirectory) {
    let files: [string, Statement, Object][] = [
        ["agency.txt", addAgency, {}],
        ["routes.txt", addRoutes, {}],
        ["calendar.txt", addCalendar, {}],
        ["calendar_dates.txt", addCalendarDates, {}],
        ["trips.txt", addTrips, {"trip_headsign": null}],
        ["stop_times.txt", addStopTimes, {}]
    ]
    let startTime = Date.now()
    for(const tuple of files) {
        await import_txt_file(zip, tuple[0], tuple[1], tuple[2])
    }
    console.log(`Insertions completed in ${(Date.now() - startTime) / 1000} seconds`)
}

async function import_txt_file(zip: CentralDirectory, file_name: string, sql: Statement, defaults: Object = {}) {
    let file = zip.files.find(f => f.path == file_name)
    if(file) {
        await insertSource(file, sql, defaults)
    } else throw Error(`Could not find ${file_name}`)
}

async function insertSource(file: File, sql: Statement, defaults: Object = {}) {
    let buffer = []
    let batchInsert = db.transaction((records) => {
        for(const record of records) {
            sql.run(record)
        }
    })
    for await (const record of stream_csv(file)) {
        Object.entries(defaults).forEach(([k, v]) => {
            if(!(k in record)) record[k] = v
        })
        buffer.push(record)
        if(buffer.length >= 100000) {
            try {
                batchInsert(buffer)
            } catch (e) {
                throw Error(`Insertion error: ${file.path} using ${sql.source}. ${e instanceof Error ? e.toString() : ""}`)
            }
            buffer = []
        }
    }
    if(buffer.length > 0) {
        try {
            batchInsert(buffer)
        } catch (e) {
            throw Error(`Insertion error: ${file.path} using ${sql.source}. ${e instanceof Error ? e.toString() : ""}`)
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

function clean_arrivals() {
    console.log("Cleaning arrivals")
    let times = select_all.all()
    const times_length = times.length
    times = times.filter(val => !(!val['arrival_time'] || !val['dep_id'] || !val['arr_id']))
    console.log(`Fixing ${times.length} arrivals (could not fix ${times_length - times.length})`)
    db.transaction((values) => {
        for(const val of values) {
            copy_one.run(val['arrival_time'], val['dep_id'])
            del_one.run(val['arr_id'])
        }
    })(times)
}

function clean_stops() {
    console.log("Removing stops with no departures")
    db.pragma("foreign_keys = OFF")
    db.exec("DELETE FROM stops WHERE stops.id NOT IN (SELECT DISTINCT stances.stop FROM stop_times INNER JOIN stances ON stances.code=stop_id) AND (NOT EXISTS(SELECT 1 FROM stances WHERE stances.stop=stops.id AND crs IS NOT NULL));");
    console.log("Rebuilding stop search table")
    // Rebuild stops_search table
    db.exec("DROP TABLE IF EXISTS stops_search;");
    db.exec("CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED, locality UNINDEXED);");
    db.exec("INSERT INTO stops_search(name, parent, qualifier, id, locality) SELECT stops.name, stops.locality_name, qualifier, stops.id, stops.locality FROM stops INNER JOIN localities l on l.code = stops.locality;");
}

function clean_sequence_numbers() {
    let startTime = Date.now()
    console.log("Updating min stop sequence numbers")
    db.exec("UPDATE trips SET min_stop_seq=(SELECT min(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)")
    console.log(`min stop sequence number updates finished in ${(Date.now() - startTime) / 1000} seconds`)

    startTime = Date.now()
    console.log("Updating max stop sequence numbers")
    db.exec("UPDATE trips SET max_stop_seq=(SELECT max(stop_sequence) FROM stop_times WHERE stop_times.trip_id=trips.trip_id)")
    console.log(`Max stop sequence number updates finished in ${(Date.now() - startTime) / 1000} seconds`)
}

function create_indexes() {
    console.log("Creating indexes")
    db.exec(indexingScript)
}

await import_zips()
create_indexes()
clean_sequence_numbers()
clean_arrivals()
clean_stops()
await downloadRouteDirections()
db.close()