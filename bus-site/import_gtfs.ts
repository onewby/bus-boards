import type {Statement} from "better-sqlite3";
import Database from "better-sqlite3";
import {parse as parsePath} from "node:path";
import {fileURLToPath} from "node:url";
import {createWriteStream, readFileSync, writeFileSync} from "fs";
import {Readable} from 'stream';
import {finished} from 'stream/promises';
import type {CentralDirectory, File} from "unzipper";
import {Open} from "unzipper";
import {parse} from "csv-parse";
import hasha from "hasha";
import JSON5 from "json5";
import {existsSync, rmSync} from "node:fs";
import groupBy from "object.groupby";
import compare from "string-comparison"
import {Client} from "basic-ftp";
import polyline from "google-polyline"

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
let addCalendar = db.prepare("REPLACE INTO calendar (service_id, start_date, end_date, validity) VALUES (:service_id, :start_date, :end_date, (:monday + (:tuesday << 1) + (:wednesday << 2) + (:thursday << 3) + (:friday << 4) + (:saturday << 5) + (:sunday << 6)))")
let addCalendarDates = db.prepare("REPLACE INTO calendar_dates (service_id, date, exception_type) VALUES (:service_id, :date, :exception_type)")
let addTrips = db.prepare("REPLACE INTO trips (route_id, service_id, trip_id, trip_headsign, shape_id) VALUES (:route_id, :service_id, :trip_id, :trip_headsign, :shape_id)")
let addStopTimes = db.prepare("REPLACE INTO stop_times (trip_id, arrival_time, departure_time, stop_id, stop_sequence, timepoint, stop_headsign, pickup_type, drop_off_type) VALUES (:trip_id, substr(:arrival_time, 1, 2)*3600+substr(:arrival_time, 4, 2)*60+substr(:arrival_time, 7, 2), substr(:departure_time, 1, 2)*3600+substr(:departure_time, 4, 2)*60+substr(:departure_time, 7, 2), :stop_id, :stop_sequence, :timepoint, NULLIF(:stop_headsign, ''), :pickup_type, :drop_off_type)")
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
    const prefixable = ["service_id", "trip_id", "shape_id"]
    let files: [string, Statement, Object?, string[]?][] = [
        ["agency.txt", addAgency],
        ["routes.txt", addRoutes, {"agency_id": "Ember"}],
        ["calendar.txt", addCalendar, {}, prefixable],
        ["calendar_dates.txt", addCalendarDates, {}, prefixable],
        ["trips.txt", addTrips, {"trip_headsign": null, "shape_id": null}, prefixable],
        ["stop_times.txt", addStopTimes, {"stop_headsign": null}]
    ]
    let startTime = Date.now()
    for(const tuple of files) {
        await import_txt_file(zip, tuple[0], tuple[1], tuple?.[2], tuple?.[3], prefix)
    }
    let shapeFile: File | undefined = zip.files.find(f => f.path === "shapes.txt")
    if(shapeFile) {
        await import_shapes(shapeFile, prefix)
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

const storeShape = db.prepare("INSERT INTO shapes (shape_id, polyline) VALUES (?, ?)")
const storeShapes = db.transaction((shapes: [string, string][]) => {
    for(let shape of shapes) {
        storeShape.run(...shape)
    }
})

// Encoded then imported
async function import_shapes(shapeFile: File, prefix?: string) {
    let currentShapeID = ""
    let currentShapePoints: [number, number][] = []
    let shapesToWrite: [string, string][] = []
    for await (const row of stream_csv(shapeFile)) {
        if(row.shape_id !== currentShapeID) {
            if(currentShapeID !== "") {
                shapesToWrite.push([prefix ? prefix + currentShapeID : currentShapeID, polyline.encode(currentShapePoints)])
                if(shapesToWrite.length >= 1000) {
                    storeShapes(shapesToWrite)
                    shapesToWrite = []
                }
            }
            currentShapeID = row.shape_id
            currentShapePoints = [[Number(row.shape_pt_lat), Number(row.shape_pt_lon)]]
        } else {
            currentShapePoints.push([Number(row.shape_pt_lat), Number(row.shape_pt_lon)])
        }
    }
    shapesToWrite.push([prefix ? prefix + currentShapeID : currentShapeID, polyline.encode(currentShapePoints)])
    storeShapes(shapesToWrite)
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

function minPredicate<T>(arr: T[], comparator: (i1: T, i2: T) => boolean) {
    let lowest = 0
    for(let i = 0; i < arr.length; i++) {
        if(comparator(arr[i], arr[lowest])) lowest = i
    }
    return arr[lowest]
}

const TRAVELINE_FILE = "gtfs/traveline_noc.zip"
const TNDS_USERNAME = process.env["TNDS_USERNAME"] ?? ""
const TNDS_PASSWORD = process.env["TNDS_PASSWORD"] ?? ""

const insertTraveline = db.prepare("INSERT INTO traveline (code, agency_id, website) VALUES (:code, :agency_id, :website)")
const getAgencyInfo = db.prepare(
    `SELECT agency.agency_id, agency.agency_name, (SELECT group_concat(route_short_name) FROM routes WHERE routes.agency_id=agency.agency_id ORDER BY route_short_name) as routes FROM agency`
)

type ServiceReportRecord = {
    RowId: number,
    RegionCode: string,
    RegionOperatorCode: string,
    ServiceCode: string,
    LineName: string,
    Description: string,
    StartDate: string,
    NationalOperatorCode: string,
    DataSource: string
}

type AgencyInfo = {agency_id: string, agency_name: string, routes: string}
type InsertInfo = AgencyInfo & {code?: string, website: string | null}

async function download_noc() {
    const ftp = new Client()
    let resp = await ftp.access({
        host: "ftp.tnds.basemap.co.uk",
        user: TNDS_USERNAME,
        password: TNDS_PASSWORD,
        secure: true,
        secureOptions: {
            rejectUnauthorized: false
        }
    })
    if(resp.code < 200 || resp.code >= 300) {
        console.log(resp.message)
        ftp.close()
        return
    }
    console.log("Connected to FTP")
    await ftp.downloadTo("gtfs/traveline.csv", "servicereport.csv")
    console.log("Downloaded TNDS service report")
    ftp.close()

    await download_zip("https://www.travelinedata.org.uk/wp-content/themes/desktop/nocadvanced_download.php?reportFormat=csvFlatFile&allTable%5B%5D=table_noc_table&allTable%5B%5D=table_public_name&submit=Submit", TRAVELINE_FILE)

    const zip = await Open.file(TRAVELINE_FILE)
    let file = zip.files.find(f => f.path.endsWith("PublicName.csv"))!
    // ordered by PubNmId
    let publicNameRecords: {PubNmId: string, Website: string}[] = await stream_csv(file).toArray()

    file = zip.files.find(f => f.path.endsWith("NOCTable.csv"))!
    // ordered alphabetically
    let nocTable: Record<string, {NOCCODE: string, PubNmId: string, OperatorPublicName: string, VOSA_PSVLicenseName: string}> = Object.fromEntries(Object.entries(groupBy(await stream_csv(file).toArray(), r => r.NOCCODE)).map(([k, v]) => [k, v[0]]))

    let records: ServiceReportRecord[] = await parse(readFileSync("gtfs/traveline.csv"), {encoding: "utf-8", cast: false, cast_date: false, columns: true}).toArray()
    let nocs = groupBy(records, record => record.NationalOperatorCode)
    let routeNOCs = Object.fromEntries(Object.entries(nocs).map(([key, values]) => [values.map(srr => srr.LineName).sort().join(','), key]))

    let agencyInfo = getAgencyInfo.all() as AgencyInfo[]
    let assignedCodes: InsertInfo[] = agencyInfo.map(info => ({...info, code: routeNOCs[info.routes], website: null}))

    let assignedCodeStrs = new Set(assignedCodes.filter(a => a.code).map(a => a.code))
    let remainingAgencies = assignedCodes.filter(a => !a.code)
    let remainingTraveline: [string, string][] = Object.entries(nocs).filter(([k, v]) => !assignedCodeStrs.has(k))
        .map(([k,v]) => [k, v.map(r => r.LineName).sort().join(',')])

    for(let agency of remainingAgencies) {
        let cands = remainingTraveline.filter(([tK, tV]) => agency.agency_name === nocTable[tK]?.OperatorPublicName)
        if(cands.length === 0) {
            let lastDitchCands = Object.values(nocTable).filter(t => agency.agency_name === t.OperatorPublicName)
            if(lastDitchCands.length > 1) lastDitchCands = lastDitchCands.filter(t => t.VOSA_PSVLicenseName)
            if(lastDitchCands.length === 1) {
                agency.code = lastDitchCands[0].NOCCODE
            } else {
                console.log("Could not map", agency.agency_name, "to Traveline data")
            }
            continue
        }
        let closest = minPredicate(cands.map(([tK, tV]): [string, number] => [tK, compare.diceCoefficient.similarity(agency.routes, tV)]), (a1, a2) => a1[1] > a2[1])
        agency.code = closest[0]
    }

    // Remove duplicates, map websites
    let finalInfo = Object.values(groupBy(assignedCodes.filter(a => a.code), a => a.code!)).map(as => {
        let assignment = as[0]
        // Binary searches possible here, but the tables are small so it's not worth bothering
        let noc = nocTable[assignment.code!]
        if(!noc) return assignment
        let pnr = publicNameRecords.find(t => t.PubNmId === noc!.PubNmId)
        if(!pnr) return assignment

        let firstIndex = pnr?.Website?.indexOf('#')
        let lastIndex = pnr?.Website?.lastIndexOf('#')
        assignment.website = pnr?.Website && firstIndex >= 0 && lastIndex >= 0 && firstIndex !== lastIndex ? pnr.Website.substring(firstIndex + 1, lastIndex) : pnr.Website
        return assignment
    })

    db.exec("DELETE FROM traveline")
    db.transaction(records => {
        records.forEach((record: any) => {
            insertTraveline.run(record)
        })
    })(finalInfo)
}

function remove_traveline_ember() {
    db.exec("DELETE FROM stop_times WHERE trip_id=(SELECT trip_id FROM trips INNER JOIN main.routes r on r.route_id = trips.route_id WHERE r.agency_id='OP965')")
    db.exec("DELETE FROM trips WHERE trip_id=(SELECT trip_id FROM trips INNER JOIN main.routes r on r.route_id = trips.route_id WHERE r.agency_id='OP965')")
    db.exec("DELETE FROM routes WHERE agency_id='OP965'")
}

await import_zips()
create_indexes()
clean_sequence_numbers()
clean_arrivals()
clean_stops()
reset_polar()
patch_display_names()
await download_noc()
db.close()