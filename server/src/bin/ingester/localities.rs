use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::str::FromStr;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use spex::parsing::XmlReader;
use spex::xml::Element;
use BusBoardsServer::download_if_old;

pub type Localities = HashMap<LocalityCode, HashMap<StopName, Vec<Stance>>>;
pub type LocalityCode = String;
pub type StopName = String;

const NAPTAN_NS: &str = "http://www.naptan.org.uk/";

#[derive(Serialize, Deserialize, Clone)]
pub struct Stance {
    #[serde(rename = "ATCOCode")]
    pub atco_code: String,
    #[serde(rename = "Lat")]
    pub lat: f64,
    #[serde(rename = "Long")]
    pub long: f64,
    #[serde(rename = "Street")]
    pub street: Option<String>,
    #[serde(rename = "Indicator")]
    pub indicator: Option<String>,
    #[serde(rename = "Arrival")]
    pub arrival: bool,
    #[serde(rename = "CrsRef")]
    pub crs: Option<String>
}

pub fn load_localities_json() -> Localities {
    let json_str = fs::read_to_string("localities.json").expect("Cannot find localities.json");
    serde_json::from_str(&json_str).expect("JSON parse fail")
}

pub fn insert_localities(db: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Insert localities into database");
    let nptg_file = download_if_old("https://naptan.api.dft.gov.uk/v1/nptg", "NPTG.xml")?;
    let xml = XmlReader::parse_auto(nptg_file)?;
    let mut xml_localities = xml.root()
        .pre_ns(NAPTAN_NS)
        .all("NptgLocalities");
    let localities = xml_localities.all("NptgLocality").iter().filter_map(convert_locality);

    db.pragma_update(None, "foreign_keys", "OFF")?;
    let tx = db.transaction()?;
    {
        let mut stmt = tx.prepare("REPLACE INTO localities (code, name, qualifier, parent, long, lat) VALUES (?, ?, ?, ?, ?, ?)")?;
        localities.for_each(|loc| {
            stmt.execute(params![
                loc.code, loc.name, loc.qualifier, loc.parent, loc.long, loc.lat
            ]).expect("Error inserting locality");
        });
    }
    tx.commit()?;
    db.pragma_update(None, "foreign_keys", "ON")?;
    db.execute_batch(r"
        DROP TABLE IF EXISTS localities_search;
        CREATE VIRTUAL TABLE localities_search USING fts5(name, qualifier, code UNINDEXED);
        INSERT INTO localities_search(name, qualifier, code) SELECT name, qualifier, code FROM localities;
    ")?;
    Ok(())
}

fn convert_locality(locality: &Element) -> Option<Locality> {
    Some(Locality {
        code: locality.req(("NptgLocalityCode", NAPTAN_NS)).text().ok().unwrap().to_string(),
        name: locality.pre_ns(NAPTAN_NS).req("Descriptor").req("LocalityName").text().ok().unwrap().to_string(),
        qualifier: locality.pre_ns(NAPTAN_NS).req("Descriptor").req("Qualify").req("QualifierName").text().ok().map(|s| s.to_string()),
        parent: locality.pre_ns(NAPTAN_NS).req("ParentNptgLocalityRef").text().ok().map(|s| s.to_string()),
        long: f64::from_str(locality.pre_ns(NAPTAN_NS).req("Location").req("Translation").req("Longitude").text().ok().unwrap().trim()).ok().unwrap(),
        lat: f64::from_str(locality.pre_ns(NAPTAN_NS).req("Location").req("Translation").req("Latitude").text().ok().unwrap().trim()).ok().unwrap(),
    })
}

pub fn insert_stops(db: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("Importing stops");
    let localities = load_localities_json();
    db.execute_batch(r"
        DELETE FROM stops;
        DELETE FROM stances;
    ")?;

    let tx = db.transaction()?;
    {
        let mut insert_stop = tx.prepare("INSERT INTO stops (name, locality) VALUES (?, ?) RETURNING id")?;
        let mut insert_stance = tx.prepare("INSERT INTO stances (code, street, indicator, lat, long, stop, crs) VALUES (?, ?, ?, ?, ?, ?, ?)")?;

        for (locality, stops) in localities {
            for (stop, stances) in stops {
                match insert_stop.query_row([&stop, &locality], |row| row.get::<_, u64>(0)) {
                    Ok(stop_id) => {
                        for stance in stances {
                            insert_stance.execute(params![
                                stance.atco_code,
                                stance.street,
                                stance.indicator,
                                stance.lat,
                                stance.long,
                                stop_id,
                                stance.crs
                            ]).expect(format!("Stance insert failed: {}", stance.atco_code).as_str());
                        }
                    },
                    Err(err) => {
                        println!("Stop insert failed: {}, {}, {}", locality, stop, err);
                    }
                }
            }
        }
    }
    tx.commit()?;

    println!("Generating locality names");
    db.execute(r"
        UPDATE stops SET locality_name =
             (SELECT GROUP_CONCAT(name, ' › ') FROM (
                WITH RECURSIVE
                    find_parent_names(level, code) AS (
                        VALUES(0, stops.locality)
                        UNION
                        SELECT level+1, parent FROM localities, find_parent_names
                        WHERE localities.code=find_parent_names.code
                    )
                SELECT name FROM localities, find_parent_names
                WHERE localities.code = find_parent_names.code
                ORDER BY level desc
        ));
    ", [])?;

    /* println!("Setting up stop search");
    db.execute_batch(r"
        DROP TABLE IF EXISTS stops_search;
        CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED, locality UNINDEXED);
        INSERT INTO stops_search(name, parent, qualifier, id, locality) SELECT stops.name, stops.locality_name, qualifier, stops.id, stops.locality FROM stops INNER JOIN localities l on l.code = stops.locality;
    ")?; */

    Ok(())
}

struct Locality {
    code: String,
    name: String,
    qualifier: Option<String>,
    parent: Option<String>,
    long: f64,
    lat: f64
}