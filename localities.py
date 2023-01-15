# Populate location database tables - run stance_grouping.py first to generate localities.json

import json
from typing import TypedDict, Dict
from xml.etree.ElementTree import *

import pandas as pd
from defusedxml.ElementTree import parse
import sqlite3


class StopData(TypedDict):
    ATCOCode: str
    Lat: float
    Long: float
    Street: str
    Indicator: str


# Parse locality XML into tuples
def convert_locality(locality):
    code = locality.findtext(".//{http://www.naptan.org.uk/}NptgLocalityCode")
    name = locality.findtext(".//{http://www.naptan.org.uk/}LocalityName")
    qualifier = locality.findtext(".//{http://www.naptan.org.uk/}QualifierName")
    parent = locality.findtext(".//{http://www.naptan.org.uk/}ParentNptgLocalityRef")
    long = locality.findtext(".//{http://www.naptan.org.uk/}Longitude")
    lat = locality.findtext(".//{http://www.naptan.org.uk/}Latitude")
    if long is not None:
        long = float(long)
    if lat is not None:
        lat = float(lat)
    return code, name, qualifier, parent, long, lat


def get_crs_codes() -> Dict[str, str]:
    df: pd.Series = pd.read_csv("crs.csv", index_col="ATCOCode")["CrsRef"]
    return df.to_dict()


def init_db(db: sqlite3.Connection):
    print("Initialise database")
    # Initialise database if it does not exist
    with open("bus-site/gtfs/model.sql") as file:
        script = " ".join(file.readlines())
        db.executescript(script)


def insert_localities(db: sqlite3.Connection):
    print("Insert localities into database")

    # Import NPTG locality data and parse XML
    tree: ElementTree = parse("NPTG.xml")
    root = tree.getroot()
    localities = root.findall(".//{http://www.naptan.org.uk/}NptgLocality")

    # Parse localities into objects
    data = [convert_locality(locality) for locality in localities]

    # Add localities to the database
    db.executemany("REPLACE INTO localities (code, name, qualifier, parent, lat, long) VALUES (?, ?, ?, ?, ?, ?)", data)
    db.execute("DROP TABLE IF EXISTS localities_search")
    db.execute("CREATE VIRTUAL TABLE localities_search USING fts5(name, qualifier, code UNINDEXED)")
    db.execute("INSERT INTO localities_search(name, qualifier, code) SELECT name, qualifier, code FROM localities")


def insert_stops(db: sqlite3.Connection):
    print("Load CSV codes")
    crs_codes = get_crs_codes()

    print("Importing stops")

    # Open the stop data generated by stance_grouping.py
    with open("localities.json", "r") as stops_file:
        stops_data: dict[str, dict[str, list[StopData]]] = json.load(stops_file)

    # Clear existing data
    db.execute("DELETE FROM stops")
    db.execute("DELETE FROM stances")

    # Import stops
    for locality, stops in stops_data.items():
        for stop, stances in stops.items():
            stop_id = db.execute("INSERT INTO stops (name, locality) VALUES (?, ?) RETURNING id", (stop, locality)).fetchone()[0]
            for stance in stances:
                code = stance["ATCOCode"]
                lat = stance["Lat"]
                long = stance["Long"]
                street = stance["Street"]
                indicator = stance["Indicator"]
                crs = crs_codes.get(code)
                db.execute("INSERT INTO stances (code, street, indicator, lat, long, stop, crs) VALUES (?, ?, ?, ?, ?, ?, ?)", (code, street, indicator, lat, long, stop_id, crs))

    print("Generating locality names")

    # Generate locality names based on locality hierarchy
    db.execute("""UPDATE stops SET locality_name =
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
    """)

    print("Setting up stop search")

    # Reset and populate the stop search database
    db.execute("DROP TABLE IF EXISTS stops_search")
    db.execute("CREATE VIRTUAL TABLE stops_search USING fts5(name, parent, qualifier, id UNINDEXED)")
    db.execute("INSERT INTO stops_search(name, parent, qualifier, id) SELECT stops.name, stops.locality_name, qualifier, stops.id FROM stops INNER JOIN localities l on l.code = stops.locality")


def main():
    # Connect to stops database
    db = sqlite3.connect("bus-site/stops.sqlite")

    print("Initialise database")
    # Initialise database if it does not exist
    with open("bus-site/gtfs/model.sql") as file:
        script = " ".join(file.readlines())
        db.executescript(script)

    insert_localities(db)
    insert_stops(db)

    print("Committing to database")
    # Commit to the database
    db.commit()
    db.close()


if __name__ == "__main__":
    main()
