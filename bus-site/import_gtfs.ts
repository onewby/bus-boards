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
import {existsSync, rmSync} from "node:fs";

const sourceFiles: { name: string, url: string, path: string, prefix: string }[] = [
    {
        name: "BODS (approx ~550MB)",
        prefix: "",
        url: "https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/",
        path: "gtfs/itm_all_gtfs.zip"
    },
    {
        name: "Ember",
        prefix: "E",
        url: "https://api.ember.to/v1/gtfs/static/",
        path: "gtfs/ember.zip"
    }
]

const __file = parsePath(fileURLToPath(import.meta.url))

const db = new Database(__file.dir + "/stops.sqlite");
db.pragma('journal_mode = WAL');
db.pragma('foreign_keys = ON')

let indexingScript = readFileSync("gtfs/indexes.sql", {encoding: "utf-8"})
let tableCreateScript = readFileSync("gtfs/model.sql", {encoding: "utf-8"})
db.exec(tableCreateScript)

const gtfsTables = ["stop_times", "trips", "calendar_dates", "calendar", "routes", "agency"]
let addAgency = db.prepare("REPLACE INTO agency (agency_id, agency_name, agency_url, agency_timezone, agency_lang) VALUES (:agency_id, :agency_name, :agency_url, :agency_timezone, :agency_lang)")
let addRoutes = db.prepare("REPLACE INTO routes (route_id, agency_id, route_short_name, route_long_name, route_type) VALUES (:route_id, :agency_id, :route_short_name, :route_long_name, :route_type)")
let addCalendar = db.prepare("REPLACE INTO calendar (service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date) VALUES (:service_id, :monday, :tuesday, :wednesday, :thursday, :friday, :saturday, :sunday, :start_date, :end_date)")
let addCalendarDates = db.prepare("REPLACE INTO calendar_dates (service_id, date, exception_type) VALUES (:service_id, :date, :exception_type)")
let addShapes = db.prepare("REPLACE INTO shapes (shape_id, shape_pt_lat, shape_pt_lon, shape_pt_sequence) VALUES (:shape_id, :shape_pt_lat, :shape_pt_lon, :shape_pt_sequence)")
let addTrips = db.prepare("REPLACE INTO trips (route_id, service_id, trip_id, trip_headsign, shape_id) VALUES (:route_id, :service_id, :trip_id, :trip_headsign, :shape_id)")
let addStopTimes = db.prepare("REPLACE INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, timepoint, stop_headsign, pickup_type, drop_off_type) VALUES (:trip_id, :arrival_time, :departure_time, :stop_id, :stop_sequence, :timepoint, NULLIF(:stop_headsign, ''), :pickup_type, :drop_off_type)")
let dropAllSource = db.transaction(() => {
    db.pragma('foreign_keys = OFF')
    gtfsTables.forEach(table => db.prepare(`DELETE FROM ${table}`).run())
    db.exec(tableCreateScript)
    db.pragma('foreign_keys = ON')
})

async function import_zips() {
    if(process.argv.length < 3 || process.argv[2].toLowerCase() != "-s") {
        for(let source of sourceFiles) {
            console.log(`Downloading ${source.name} GTFS data`)
            await download_zip(source.url, source.path)
        }

        if(process.argv.length < 3 || process.argv[2].toLowerCase() != "-h") {
            const hash = await calculate_joint_hash()
            if(existsSync("gtfs/hash.txt")) {
                let currentHash = readFileSync("gtfs/hash.txt", "utf-8")
                if(currentHash === hash) {
                    console.log("GTFS up to date - run ts-node-esm import_gtfs.ts -h to ignore hash")
                }
            }
        }
    }

    console.log("Dropping previous data")
    dropAllSource()
    console.log("Inserting new data")

    for(let source of sourceFiles) {
        const zip = await Open.file(source.path)
        try {
            await import_zip(zip, source.prefix)
        } catch (e) {
            console.log(e)
        }
    }
    writeFileSync("gtfs/hash.txt", await calculate_joint_hash())

    console.log("Imported data successfully.")
}

async function download_zip(url: string, path: string) {
    let resp = await fetch(url)
    if(!resp.ok || !resp.body) {
        console.error(`Cannot download ${path}`)
        return
    }
    let fileStream = createWriteStream(path)
    // @ts-ignore
    await finished(Readable.fromWeb(resp.body).pipe(fileStream))
    fileStream.close()
}

async function calculate_joint_hash() {
    return hasha(
        (await Promise.all(
            sourceFiles.map(async source => await hasha.fromFile(source.path, {algorithm: "sha256"}))
        )).join()
    )
}

async function import_zip(zip: CentralDirectory, prefix: string = "") {
    const prefixable = ["service_id"]
    let files: [string, Statement, Object?, string[]?][] = [
        ["agency.txt", addAgency],
        ["routes.txt", addRoutes, {"agency_id": "Ember"}],
        ["calendar.txt", addCalendar, {}, prefixable],
        ["calendar_dates.txt", addCalendarDates, {}, prefixable],
        ["shapes.txt", addShapes],
        ["trips.txt", addTrips, {"trip_headsign": null, "shape_id": null}, prefixable],
        ["stop_times.txt", addStopTimes, {"stop_headsign": null}]
    ]
    let startTime = Date.now()
    for(const tuple of files) {
        await import_txt_file(zip, tuple[0], tuple[1], tuple?.[2], tuple?.[3], prefix)
    }
    console.log(`Insertions completed in ${(Date.now() - startTime) / 1000} seconds`)
}

async function import_txt_file(zip: CentralDirectory, file_name: string, sql: Statement, defaults: Object = {}, prefixable: string[] = [], prefix: string = "") {
    let file = zip.files.find(f => f.path == file_name)
    if(file) {
        await insertSource(file, sql, defaults, prefixable, prefix)
    } else throw Error(`Could not find ${file_name}`)
}

async function insertSource(file: File, sql: Statement, defaults: Object = {}, prefixable: string[] = [], prefix="") {
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
        if(prefix) prefixable.forEach(field => {
            record[field] = prefix + record[field]
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
    let times = select_all.all() as {arrival_time: string, arr_id: string, dep_id: string}[]
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

function patch_display_names() {
    console.log("Patching route display names")
    // Change Ember route names from 'Ember' to E{1,3,10}
    db.exec("UPDATE routes SET route_short_name=route_id WHERE agency_id='Ember'")
    // Fix coach operators' destinations where local locality names used
    db.exec(`UPDATE trips SET trip_headsign=(SELECT CASE
    WHEN original = 'Tokyngton' THEN 'Wembley Stadium'
    WHEN original = 'Centenary Square' THEN 'Birmingham'
    WHEN original = 'Penglais' THEN 'Aberystwyth University'
    WHEN original = 'Causewayhead' THEN 'University of Stirling'
    WHEN origin_loc = dest_loc THEN original
    WHEN dest_loc = 'London' THEN dest_loc || ' ' || original
    ELSE dest_loc END)
FROM (SELECT trips.trip_id,
         (SELECT IFNULL(substr(s.locality_name, 0, NULLIF(instr(s.locality_name, '›') - 1, -1)), s.locality_name)
            FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=dest.stop_id) as dest_loc,
         (SELECT IFNULL(substr(s.locality_name, 0, NULLIF(instr(s.locality_name, '›') - 1, -1)), s.locality_name)
            FROM stances INNER JOIN main.stops s on s.id = stances.stop WHERE code=origin.stop_id) as origin_loc,
          trip_headsign AS original FROM trips
  INNER JOIN main.routes r on r.route_id = trips.route_id
  INNER JOIN main.stop_times origin on (trips.trip_id = origin.trip_id AND trips.min_stop_seq=origin.stop_sequence)
  INNER JOIN main.stop_times dest on (trips.trip_id = dest.trip_id AND trips.max_stop_seq=dest.stop_sequence)
WHERE agency_id IN ('OP5050', 'OP564', 'OP5051', 'OP545', 'OP563') AND NOT instr(trip_headsign, 'Airport')) AS trip_subquery
WHERE trips.trip_id=trip_subquery.trip_id`)
}

// Remove .update file so Passenger trip IDs are remapped at runtime
function reset_polar() {
    rmSync(".update", {force: true})
}

type NOCRecord = {
    code: string,
    agencyID: string,
    website: string
}

const getAgencyID = db.prepare("SELECT agency_id FROM agency WHERE agency_name=?").pluck()
const insertTraveline = db.prepare("INSERT INTO traveline (code, agency_id, website) VALUES (:code, :agency_id, :website)")

async function download_noc() {
    await download_zip("https://www.travelinedata.org.uk/wp-content/themes/desktop/nocadvanced_download.php?reportFormat=csvFlatFile&submit=Submit", "gtfs/traveline_noc.zip")
    const zip = await Open.file("gtfs/traveline.zip")

    let file = zip.files.find(f => f.path.endsWith("PublicName.csv"))!
    let publicNameRecords = await stream_csv(file).toArray()
    file = zip.files.find(f => f.path.endsWith("NOCLines.csv"))!
    let nocLines = await stream_csv(file).toArray()

    file = zip.files.find(f => f.path.endsWith("NOCTable.csv"))!

    let map: Record<string, NOCRecord[]> = {}

    for await (const record of stream_csv(file)) {
        if(!record.NOCCODE || !record.OperatorPublicName || !record.PubNmId) continue

        let pnr = publicNameRecords.find(r => r.PubNmId === record.PubNmId)
        let firstIndex = pnr.website.indexOf('#')
        let lastIndex = pnr.website.lastIndexOf('#', firstIndex)
        let website = firstIndex >= 0 && lastIndex >= 0 ? pnr.website.substring(firstIndex + 1, lastIndex) : pnr.website

        if(map[record.OperatorPublicName] === undefined) map[record.OperatorPublicName] = []
        map[record.OperatorPublicName].push({
            code: record.NOCCODE,
            agencyID: "",
            website
        })
    }

    for(let [name, entries] of Object.entries(map)) {
        if(entries.length === 1) {
            let aid = getAgencyID.get(name) as string | undefined
            if(aid !== undefined) {
                entries[0].agencyID = aid
                //insertTraveline.run(entries[0])
            } else {
                console.log("Could not find agency: " + entries[0])
            }
        } else {
            let possibilities = nocLines.filter(r => r["PubNm"] === name)
            let agencyIDs = getAgencyID.all(name) as string[]
            if(possibilities.length !== agencyIDs.length) {
                possibilities = possibilities.filter(r => Object.entries(r).some(([name, val]) => name !== "NOCCODE" && val === r["NOCCODE"]))
            }
            if(possibilities.length === agencyIDs.length) {
                // sort alphabetically, shift those with numbers to the end
                possibilities.sort()
                possibilities = [...possibilities.filter(p => !/\d/.test(p.OPCODE)), possibilities.filter(p => /\d/.test(p.OPCODE))]
                let finalEntries: NOCRecord[] = []
                let i = 0
                for (const p of possibilities) {
                    let entry = entries.find(r => r.code === p.OPCODE)
                    if(entry) {
                        entry.agencyID = agencyIDs[i]
                        finalEntries.push(entry)
                        i++
                        if(i == entries.length) break
                    }
                }
                /*db.transaction(record => {
                    insertTraveline.run(record)
                })(finalEntries)*/
            } else {
                console.log("Could not assign to agency:", entries)
            }
        }
    }
}

await import_zips()
create_indexes()
clean_sequence_numbers()
clean_arrivals()
clean_stops()
reset_polar()
patch_display_names()
db.close()