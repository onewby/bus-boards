# Generate stops by grouping stances in the same location together

import json
import re
import pandas as pd
from typing import TypedDict, List, Dict, Tuple
import pyproj
import numpy as np


class Stop(TypedDict):
    ATCOCode: str
    NaptanCode: str
    PlateCode: str
    CleardownCode: str
    CommonName: str
    CommonNameLang: str
    ShortCommonName: str
    ShortCommonNameLang: str
    Landmark: str
    LandmarkLang: str
    Street: str
    StreetLang: str
    Crossing: str
    CrossingLang: str
    Indicator: str
    IndicatorLang: str
    Bearing: str
    NptgLocalityCode: str
    LocalityName: str
    ParentLocalityName: str
    GrandParentLocalityName: str
    Town: str
    TownLang: str
    Suburb: str
    SuburbLang: str
    LocalityCentre: str
    GridType: str
    Easting: int
    Northing: int
    Longitude: float
    Latitude: float
    StopType: str
    BusStopType: str
    TimingStatus: str
    DefaultWaitTime: str
    Notes: str
    NotesLang: str
    AdministrativeAreaCode: int
    CreationDateTime: str
    ModificationDateTime: str
    RevisionNumber: int
    Modification: str
    Status: str


class StopGroup(TypedDict):
    name: str
    stops: List[Stop]


class Stance(TypedDict):
    ATCOCode: str
    Indicator: str
    Street: str
    Lat: float
    Long: float


LocalityName = str
StopName = str
StopGroupings = Dict[LocalityName, Dict[StopName, List[Stance]]]
LocalityCode = str
StopName = str

locality_changes: List[Tuple[LocalityCode, LocalityName, LocalityCode, LocalityCode]] = [
    ("N0077860", "Park Lane (Tyne and Wear Metro Station)", "E0057917", np.nan),
    ("E0057948", "Bradford Interchange Rail Station", "N0077005", "Bradford"),
    ("E0057974", "Leeds Rail Station", "N0077039", "Leeds"),
    ("ES003919", "Dundee Rail Station", "ES000536", "Dundee")
]

# (locality, from name, to name)
manual_renames: List[Tuple[LocalityCode, StopName, StopName]] = [
    ("N0078622", "Edinburgh Airport (Edinburgh Trams)", "Airport"),
    ("ES001737", "Haymarket (Edinburgh Trams)", "Haymarket Station"),
    ("ES001737", "Haymarket Station", "Rail Station"),
    ("E0049583", "Stand 4", "Bus Station"),  # Tadcaster
    ("ES000536", "Bus Station", "Seagate Bus Station"),  # Dundee
    ("N0078275", "Edinburgh Park Station", "Edinburgh Park Rail Station"),
    ("N0078275", "Edinburgh Park Station (Edinburgh Trams)", "Edinburgh Park Rail Station"),
    ("E0057900", "Newcastle Rail Station", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station (Tyne and Wear Metro Station)", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station Bewick Street", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station Clayton St", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station Neville St", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station Westgate Rd", "Newcastle Central Rail Station"),
    ("E0057900", "Central Station Westgate Road", "Newcastle Central Rail Station"),
    ("N0078208", "Ponteland Road - Newcastle Airport", "Newcastle Airport Ponteland Road"),
    ("N0078208", "Newcastle Airport Metro Station", "Newcastle Airport (Tyne and Wear Metro Station)"),
    ("E0055009", "Hull Rail Station", "Hull Paragon Interchange (Rail Station)"),
    ("E0055009", "Hull Interchange", "Hull Paragon Interchange (Rail Station)"),
    ("N0077005", "Bradford Interchange Rail Station", "Interchange"),
    ("E0057917", "Sunderland Interchange", "Park Lane Interchange"),
    ("E0057917", "Park Lane (Tyne and Wear Metro Station)", "Park Lane Interchange"),
    ("E0057917", "Sunderland (Tyne and Wear Metro Station)", "Rail Station"),
    ("ES000536", "Station", "Rail Station")
]

# ATCOCodes for stances that should be marked as arrivals that are not
manual_arrivals: List[str] = [
    "6400L00040"
]


def group_data(df: pd.DataFrame) -> StopGroupings:
    # print("Dropping unwanted entries")
    # df["NaptanCode"].replace("", np.nan, inplace=True)

    crs = pd.read_csv("crs.csv", index_col="ATCOCode")
    df = df.merge(crs, on="ATCOCode", how='left')
    # df.loc[df["CrsRef"].notna(), "NaptanCode"] = ""
    #
    # df.dropna(subset=["NaptanCode"], inplace=True)

    print("Converting easting/northing to lat/long")
    # https://gist.github.com/amercader/927079/caa63f49d1ff36f0489f2e11cb695deb06d5b6c2
    transformer = pyproj.Transformer.from_crs("EPSG:27700", "EPSG:4326")
    converted = transformer.transform(df["Easting"].values, df["Northing"].values)
    df["Lat"] = converted[0]
    df["Long"] = converted[1]

    print("Standardising synonyms and merging arrival bays")
    standardise_synonyms(df)

    print("Grouping data")
    return {localityCode: {stopName: stances[["ATCOCode", "Indicator", "Street", "Lat", "Long", "Arrival", "CrsRef"]].to_dict(orient="records") for stopName, stances in stops.groupby("CommonName")} for localityCode, stops in df.groupby("NptgLocalityCode")}


def standardise_synonyms(df: pd.DataFrame):
    # Denote an arrival bay
    df["Arrival"] = df["CommonName"].str.contains("Arrival", case=False) | df["Indicator"].str.contains("Arrival", case=False)
    # If an arrival bay contains info about the stances it arrives at, keep it, else discard the info
    df.loc[df["Arrival"] & df["Indicator"].isin(["at", "", None]), "Indicator"] = "Arrival Bay"
    # Merge arrival bays into their likely main stations
    df["CommonName"] = df["CommonName"].str.replace(" Arrival Bay", "")
    df["CommonName"] = df["CommonName"].str.replace("(?i) Arrivals?", "", regex=True)
    # Integrate manual fixes specified
    for change in locality_changes:
        loc_filter = (df["NptgLocalityCode"] == change[0]) & (df["CommonName"] == change[1])
        df.loc[loc_filter, "NptgLocalityCode"] = change[2]
        df.loc[loc_filter, "ParentLocalityName"] = change[3]
    for rename in manual_renames:
        loc_filter = df["NptgLocalityCode"] == rename[0]
        df.loc[loc_filter, "CommonName"] = df.loc[loc_filter, "CommonName"].replace(rename[1], rename[2])
    df.loc[df["ATCOCode"].isin(manual_arrivals), "Arrival"] = True
    # Fix Harrogate having an odd mix of Bus Stn and Hgte Bus Stn
    df["CommonName"] = df["CommonName"].str.replace("Hgte ", "")
    # Standardise the phrase "Bus Station" - fixes inconsistencies e.g. in Beverley
    df["CommonName"] = df["CommonName"].str.replace("(?i) BS", " Bus Station", regex=True)
    df["CommonName"] = df["CommonName"].str.replace("(?i)Bus Stn", "Bus Station", regex=True)
    df["CommonName"] = df["CommonName"].str.replace("(?i)Bus Station", "Bus Station", regex=True)
    # Standardise Park and Ride
    df["CommonName"] = df["CommonName"].str.replace("(?i)Park[ -/]?(?:[&+/]|and)[ -/]?Ride", "Park and Ride", regex=True)
    df["CommonName"] = df["CommonName"].str.strip()
    # Deal with Cambridge The Busway having separate stops
    df["CommonName"] = df["CommonName"].str.replace("The Busway ", "", regex=True)
    # Integrate Edinburgh Trams at Ingliston Park & Ride, Edinburgh Airport and St Andrew Square
    df["CommonName"] = df["CommonName"].str.replace(" (Edinburgh Trams)", "", regex=False)
    # Attempt to integrate stations
    # Standardise naming
    df["CommonName"] = df["CommonName"].str.replace("(?i)Railway ", "Rail ", regex=True)
    df["CommonName"] = df["CommonName"].str.replace("(?i) Stn", " Station", regex=True)
    # If only instance of name in locality, attempt to remove locality name
    stn_filter = df["CrsRef"].notna() & ~df[["NptgLocalityCode", "CommonName"]].duplicated(keep=False)
    df["OriginalCommonName"] = df["CommonName"]
    df["CommonName"] = df.apply(lambda x: simplify_station_name(df, x), axis=1)

    # Check if a child locality would be more suitable for a station

    # if station still without partner
    stn_filter = df["CrsRef"].notna() & ~df[["NptgLocalityCode", "CommonName"]].duplicated(keep=False)
    # Get stops attached to a [station] stop with a CrsRef
    df["IsRail"] = df["CrsRef"].notna()
    g = df.groupby(["NptgLocalityCode", "CommonName"])
    df = df.join(g["IsRail"].any().rename("HasRail"), on=["NptgLocalityCode", "CommonName"])
    overlap = df[stn_filter].merge(df[df["BusStopType"].notna()], left_on="LocalityName", right_on="ParentLocalityName")
    # Still not perfect - uses shortened CommonNames, so fails when e.g. can't discern between Invergowrie and Dundee
    candidates = overlap[((overlap["CommonName_x"] == overlap["CommonName_y"]) | (overlap["OriginalCommonName_x"] == overlap["OriginalCommonName_y"])) & (overlap["HasRail_y"] == False)][["NptgLocalityCode_x", "CommonName_x", "OriginalCommonName_x", "LocalityName_x", "CrsRef_x", "NptgLocalityCode_y", "CommonName_y", "OriginalCommonName_y", "NptgLocalityCode_y", "LocalityName_y", "ParentLocalityName_y"]]
    # Filter out candidates where locality y already has a station
    candidates = candidates[~candidates["LocalityName_y"].isin(df[df["IsRail"]]["LocalityName"])]
    # There's not many of these to set, so do this the simple way
    for index, candidate in candidates.iterrows():
        df.iat[index, df.columns.get_loc("CommonName")] = candidate["CommonName_y"]
        df.iat[index, df.columns.get_loc("NptgLocalityCode")] = candidate["NptgLocalityCode_y"]


# Patterns to group stances into stops using
stances_regex = re.compile(r'(?P<stop>.*[^-]) *[-/ ] *(?P<stance>(?:Stance |Stand |Stop ) *[a-zA-Z]?\d{0,2})', re.IGNORECASE)
interchange_regex = re.compile(r'(?P<stop>.*[^-])/(?P<stance>[a-zA-Z]?\d{0,2})')
leeds_regex = re.compile(r'(?P<stop>[a-zA-Z ]+) (?P<stance>[a-zA-Z]?\d{0,2})')
station_regex = re.compile(r'(?P<stop>[a-zA-Z]+) (?P<stance>[a-zA-Z]\d{1,2})')


def simplify_station_name(df: pd.DataFrame, x: Stop):
    if x["CommonName"].endswith("Rail Station"):
        new_name = x["CommonName"].removeprefix(x["LocalityName"] + " ")
        return new_name if (df["CommonName"] == new_name).any() else x["CommonName"]
    else:
        return x["CommonName"]


# Split a stop name into a stop location and stance
def split_stop_name(stop: str):
    pattern = stances_regex.fullmatch(stop)
    if pattern is None:
        pattern = interchange_regex.fullmatch(stop)
        if pattern is None:
            pattern = leeds_regex.fullmatch(stop)
            if pattern is None:
                pattern = station_regex.fullmatch(stop)
    if pattern is None or pattern.group("stop") == "Number" or "H&R" in pattern.group("stop") or "H & R" in pattern.group("stop") or pattern.group("stop").endswith("No") or pattern.group("stop").endswith("No.") or pattern.group("stop").endswith("Unit"):
        return None
    return {"original": stop, "stop": pattern.group("stop"), "stance": pattern.group("stance")}


def fix_groupings(groups: StopGroupings):
    # Handle situations like XYZ Interchange/X00 having different stops
    for locality in groups:
        # get original-stem-suffix tuples, group by matching stems, merge if 2+ stems match
        split_stops = [split_stop_name(stop) for stop in groups[locality]]
        split_stops = filter(lambda s: s is not None, split_stops)
        df = pd.DataFrame.from_records(split_stops)
        if len(df.index) == 0:
            continue
        # Group records by stop name
        stops = {k: table.to_dict(orient="records") for k, table in df.groupby("stop")}
        # For each new stop to create by merging other stops...
        for stop in stops:
            # Avoid merging false positives
            if len(stops[stop]) <= 1:
                continue
            # For each existing stop we can merge together:
            for merge_candidate in stops[stop]:
                current_name = merge_candidate["original"]
                # If the stop to merge has multiple stances associated with it
                include_original_stances = False
                if len(groups[locality][current_name]) > 1:
                    # Check to see if the stances have some meaning we shouldn't just discard:
                    def f(s: Stance): return not (s["Indicator"] == merge_candidate["stance"] or str(s["Indicator"]).isnumeric() or pd.isna(s["Indicator"]))
                    if len(list(filter(f, groups[locality][current_name]))) != 0:
                        include_original_stances = True
                # Initialise stop if it doesn't exist - don't overwrite what may already be there
                if stop not in groups[locality]:
                    groups[locality][stop] = []
                # But regardless, merge all stances associated with the stop
                for original_stop in groups[locality][current_name]:
                    if include_original_stances:
                        original_stop["Indicator"] = str(original_stop["Indicator"]) + " " + merge_candidate["stance"]
                    else:
                        original_stop["Indicator"] = merge_candidate["stance"]
                    groups[locality][stop].append(original_stop)
                # print("- Merged", stop, current_name)
                # Delete the unmerged stop
                del groups[locality][current_name]

    return groups


def main():
    print("Reading stops")
    df = pd.read_csv("Stops.csv", low_memory=False)

    groups = group_data(df)

    print("Fixing groupings")
    groups = fix_groupings(groups)

    print("Writing to file")
    with open("localities.json", "w") as file:
        json.dump(groups, file)


if __name__ == "__main__":
    main()
