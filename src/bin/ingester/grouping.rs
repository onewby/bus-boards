use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;

use geo_types::Coord;
use itertools::Itertools;
use polars::datatypes::AnyValue;
use polars::frame::DataFrame;
use polars::frame::row::Row;
use polars::io::SerReader;
use polars::prelude::{as_struct, ChainedThen, coalesce, col, cols, CsvReadOptions, DataFrameJoinOps, DataType, Expr, IntoLazy, JoinArgs, JoinType, lit, Literal, NamedFrom, not, NULL, Schema, SchemaRef, SmartString, StringNameSpaceImpl, when};
use polars::series::Series;
use proj::Proj;
use regex::{Regex, RegexBuilder};
use tower_http::compression::Predicate;

use BusBoardsServer::download_if_old;

use crate::localities::{Localities, Stance};
use crate::locality_changes::{LOCALITY_CHANGES, MANUAL_ARRIVALS, MANUAL_RENAMES};

pub fn group_stances() -> Result<Localities, Box<dyn Error>> {
    download_if_old("https://naptan.api.dft.gov.uk/v1/access-nodes?dataFormat=csv", "Stops.csv")?;
    let mut df = load_csv("Stops.csv")?;

    println!("Grouping stances");
    let mut groupings = group_data(&mut df)?;
    println!("Fixing groupings");
    fix_groupings(&mut groupings)?;
    println!("Writing to localities.json");
    write_groups(&groupings)?;
    Ok(groupings)
}

fn group_data(df: &mut DataFrame) -> Result<Localities, Box<dyn Error>> {
    println!("Loading CSV");
    let crs = load_csv("crs.csv")?;
    let mut df = df.join(&crs, ["ATCOCode"], ["ATCOCode"], JoinArgs::new(JoinType::Left))?;

    println!("Converting coordinates");
    let proj = Proj::new_known_crs("EPSG:27700", "EPSG:4326", None)?;
    let mut coords: Vec<Coord<f64>> = df.select(["Easting", "Northing"])?
        .into_struct("").iter()
        .map(|vals| Coord::from((vals[0].try_extract().unwrap(), vals[1].try_extract().unwrap()))).collect();
    proj.convert_array(&mut coords)?;
    let lat = coords.iter().map(|c| c.x).collect_vec();
    let lon = coords.iter().map(|c| c.y).collect_vec();
    df
        .with_column(Series::new("Lat", lat))?
        .with_column(Series::new("Lon", lon))?;

    println!("Standardising synonyms");
    let df = standardise_synonyms(df)?;

    println!("Grouping data");
    let location_vals = df.select(["NptgLocalityCode", "CommonName", "ATCOCode", "Indicator", "Street", "Lat", "Lon", "Arrival", "CrsRef"])?
        .into_struct("loc");
    let location_data: Vec<(&str, &str, Stance)> = location_vals.into_iter().map(|s|
            (s[0].get_str().unwrap(), s[1].get_str().unwrap(), Stance {
                atco_code: s[2].get_str().unwrap().to_string(),
                lat: s[5].try_extract().unwrap(),
                long: s[6].try_extract().unwrap(),
                street: s[4].get_str().map(|s| s.to_string()),
                indicator: s[3].get_str().map(|s| s.to_string()),
                arrival: {
                    match s[7] {
                        AnyValue::Boolean(res) => res,
                        _ => false
                    }
                },
                crs: s[8].get_str().map(|s| s.to_string()),
            }
    )).collect();

    let mut data: Localities = HashMap::new();
    location_data.iter().for_each(|(k0, k1, stance)| {
        data.entry(k0.to_string())
            .or_insert_with(HashMap::new)
            .entry(k1.to_string())
            .or_insert_with(Vec::new)
            .push(stance.clone())
    });

    Ok(data)
}

fn standardise_synonyms(df: DataFrame) -> Result<DataFrame, Box<dyn Error>> {
    let df = df.lazy()
        // Denote an arrival bay
        .with_column(
            col("CommonName").str().contains(lit("(?i)Arrival"), true)
                .or(col("Indicator").str().contains(lit("(?i)Arrival"), true))
                .alias("Arrival"))
        // If an arrival bay contains info about the stances it arrives at, keep it, else discard the info
        .with_column(
            when(col("Arrival").and(col("Indicator").str().contains(lit(r"^(at|)$"), true).or(col("Indicator").is_null())))
                .then(lit("Arrival Bay"))
                .otherwise(col("Indicator"))
                .alias("Indicator")
        )
        .collect()?;

    // Remove "Arrival Bay" from stop names (we're not interested)
    let df = df.lazy().with_column(
        col("CommonName")
            .str().replace(lit(" Arrival Bay"), lit(""), true)
            .str().replace(lit("(?i) Arrivals?"), lit(""), false)
            .alias("CommonName")
    ).collect()?;

    // Create manual fixes to locality code, parent and name
    let mut changes_schema = Schema::new();
    changes_schema.insert_at_index(0, SmartString::from("NptgLocalityCode"), DataType::String)?;
    changes_schema.insert_at_index(1, SmartString::from("CommonName"), DataType::String)?;
    changes_schema.insert_at_index(2, SmartString::from("NewNptgLocalityCode"), DataType::String)?;
    changes_schema.insert_at_index(3, SmartString::from("NewParentLocalityName"), DataType::String)?;

    let mut renames_schema = Schema::new();
    renames_schema.insert_at_index(0, SmartString::from("NptgLocalityCode"), DataType::String)?;
    renames_schema.insert_at_index(1, SmartString::from("CommonName"), DataType::String)?;
    renames_schema.insert_at_index(2, SmartString::from("NewCommonName"), DataType::String)?;

    let changes_df = DataFrame::from_rows_iter_and_schema(
        LOCALITY_CHANGES.map(|c| Row::new(vec![
            AnyValue::String(c.0),
            AnyValue::String(c.1),
            AnyValue::String(c.2),
            c.3.map_or(AnyValue::Null, |cc| AnyValue::String(cc))
        ])).iter(),
        &changes_schema
    )?;

    let renames_df = DataFrame::from_rows_iter_and_schema(
        MANUAL_RENAMES.map(|c| Row::new(vec![
            AnyValue::String(c.0),
            AnyValue::String(c.1),
            AnyValue::String(c.2)
        ])).iter(),
        &renames_schema
    )?;

    let df = df.lazy()
        .join(changes_df.lazy(),
            [col("NptgLocalityCode"), col("CommonName")],
            [col("NptgLocalityCode"), col("CommonName")],
            JoinArgs::new(JoinType::Left)
        ).join(renames_df.lazy(),
               [col("NptgLocalityCode"), col("CommonName")],
               [col("NptgLocalityCode"), col("CommonName")],
               JoinArgs::new(JoinType::Left)
        ).with_columns([
            coalesce(&[col("NewNptgLocalityCode"), col("NptgLocalityCode")]).alias("NptgLocalityCode"),
            coalesce(&[col("NewParentLocalityName"), col("ParentLocalityName")]).alias("ParentLocalityName"),
            coalesce(&[col("NewCommonName"), col("CommonName")]).alias("CommonName"),
            when(col("ATCOCode").is_in(lit(Series::new("ATCOCode", MANUAL_ARRIVALS))))
                .then(true)
                .otherwise(col("Arrival"))
                .alias("Arrival")
        ])
        .select(&[col("*").exclude(["NewNptgLocalityCode", "NewParentLocalityName"])])
        .collect()?;

    let df = df.lazy().with_column(
        col("CommonName")
            // Fix Harrogate having an odd mix of Bus Stn and Hgte Bus Stn
            .str().replace(lit("Hgte "), lit(""), true)
            // Standardise the phrase "Bus Station" - fixes inconsistencies e.g. in Beverley
            .str().replace(lit(r"(?i) BS"), lit(" Bus Station"), false)
            .str().replace(lit(r"(?i)Bus Stn"), lit("Bus Station"), false)
            .str().replace(lit(r"(?i)Bus Station"), lit("Bus Station"), false)
            // Standardise Park and Ride
            .str().replace(lit(r"(?i)Park[ -/]?(?:[&+/]|and)[ -/]?Ride"), lit("Park and Ride"), false)
            .str().strip_chars(NULL.lit())
            // Deal with Cambridge The Busway having separate stops
            .str().replace(lit("The Busway "), lit(""), true)
            // Integrate Edinburgh Trams at Ingliston Park & Ride, Edinburgh Airport and St Andrew Square
            .str().replace(lit(" (Edinburgh Trams)"), lit(""), true)
            // Manchester Metrolink
            .str().replace(lit("(Manchester Metrolink)"), lit("Metrolink Stop"), true)
            // Attempt to integrate stations
            // Standardise naming
            .str().replace(lit(r"(?i)Railway "), lit("Rail "), false)
            .str().replace(lit(r"(?i) Stn"), lit(" Station"), false)
    ).collect()?;

    let df = df.lazy().with_columns([
        col("CommonName").alias("OriginalCommonName"),
        // Simplify "X Rail Station" in X to "Rail Station"
        when(
            col("CommonName").str().ends_with(lit("Rail Station"))
                .or(col("CommonName").str().ends_with(lit("Bus Station")))
                .and(col("CommonName").str().strip_prefix(col("LocalityName") + lit(" ")).is_in(col("CommonName")))
        ).then(col("CommonName").str().strip_prefix(col("LocalityName") + lit(" ")))
            .otherwise(col("CommonName"))
            .alias("CommonName")
    ]).collect()?;

    // Standardise station name
    let new_name = col("CommonName").str().strip_suffix(lit("Station")) + lit("Rail Station"); // X Station -> X Rail Station
    let new_name_2 = new_name.clone().str().strip_prefix(col("LocalityName") + lit(" ")); // Remove X if X is the locality name
    let df = df.lazy().with_column(
        when(col("CommonName").str().ends_with(lit("Station")).and(not(col("CommonName").str().ends_with(lit("Rail Station")))))
            .then(
                when(new_name.clone().is_in(col("CommonName")))
                    .then(new_name.clone())
                    .when(not(new_name.clone().eq(new_name_2.clone())).and(new_name_2.clone().is_in(col("CommonName"))))
                    .then(new_name_2.clone())
                    .otherwise(col("CommonName"))
            )
            .otherwise(col("CommonName"))
            .alias("CommonName")
    ).collect()?;

    // Check if a child locality would be more suitable for a station
    let is_rail = col("CrsRef").is_not_null();
    let stn_filter = is_rail.clone().and(not(as_struct(vec![col("NptgLocalityCode"), col("CommonName")]).is_duplicated()));
    let df = df.lazy()
        .with_column(is_rail.clone().alias("IsRail"))
        .collect()?;
    // Mark on each stance whether it is part of a larger stop that has a rail station
    let has_rail = df.clone().lazy().group_by([col("NptgLocalityCode"), col("CommonName")])
        .agg(&[is_rail.clone().any(false).alias("HasRail")]);
    let df = df.lazy()
        .join(
            has_rail,
            [col("NptgLocalityCode"), col("CommonName")],
            [col("NptgLocalityCode"), col("CommonName")],
            JoinArgs::new(JoinType::Left))
        .collect()?;
    // Try matching station candidates to a child locality
    let candidates = df.clone().lazy().filter(stn_filter)
        .join(
            df.clone().lazy().filter(col("BusStopType").is_not_null()),
            [col("LocalityName")], [col("ParentLocalityName")],
            JoinArgs::new(JoinType::Inner).with_suffix(Some("_y".to_string())))
        .filter(
            col("CommonName").eq(col("CommonName_y"))
                .or(col("OriginalCommonName").eq("OriginalCommonName_y"))
                .and(col("HasRail_y").eq(false))
                .and(not(col("LocalityName_y").is_in(col("LocalityName").filter(col("IsRail")))))
        )
        .select(&[col("ATCOCode"), col("CommonName_y").alias("NewCommonName"), col("NptgLocalityCode_y").alias("NewNptgLocalityCode")]);
    // Override with new name/locality where warranted
    let df = df.lazy()
        .left_join(candidates, col("ATCOCode"), col("ATCOCode"))
        .with_columns([
            coalesce(&[col("NewCommonName"), col("CommonName")]).alias("CommonName")     ,
            coalesce(&[col("NewNptgLocalityCode"), col("NptgLocalityCode")]).alias("NptgLocalityCode")
        ])
        .select(&[col("*").exclude(["NewCommonName", "NewNptgLocalityCode"])])
        .collect()?;
    Ok(df)
}

// Handle situations like XYZ Interchange/X00 having different stops
fn fix_groupings(localities: &mut Localities) -> Result<(), Box<dyn Error>> {
    let stances_regex = RegexBuilder::new(r"^(?P<stop>.*[^-]) *[-/ ] *(?P<stance>(?:Stance |Stand |Stop ) *[a-zA-Z]?\d{0,2})$").case_insensitive(true).build()?;
    let interchange_regex = Regex::new(r"^(?P<stop>.*[^-])/(?P<stance>[a-zA-Z]?\d{0,2})$")?;
    let leeds_regex = Regex::new(r"^(?P<stop>[a-zA-Z ]+) (?P<stance>[a-zA-Z]?\d{0,2})$")?;
    let station_regex = Regex::new(r"^(?P<stop>[a-zA-Z]+) (?P<stance>[a-zA-Z]\d{1,2})$")?;

    let mut schema = Schema::new();
    schema.insert_at_index(0, SmartString::from("original"), DataType::String)?;
    schema.insert_at_index(1, SmartString::from("stop"), DataType::String)?;
    schema.insert_at_index(2, SmartString::from("stance"), DataType::String)?;

    let loc_keys = localities.keys().map(|k| k.to_string()).collect_vec();
    for locality in loc_keys {
        // get original-stem-suffix tuples, group by matching stems, merge if 2+ stems match
        let split_stops = localities[&locality].iter()
            .filter_map(|(s, _)| split_stop_name(s, &stances_regex, &interchange_regex, &leeds_regex, &station_regex))
            // Group by stop name
            .into_group_map_by(|(original, stop, stance)| stop.clone());
        // For each new stop to create by merging other stops...
        for (stop, merge_candidates) in split_stops {
            // Avoid merging false positives
            if merge_candidates.len() <= 1 {
                continue;
            }
            // For each existing stop we can merge together:
            for (current_name, stop, stance) in merge_candidates {
                // If the stop to merge has multiple stances associated with it
                let mut include_original_stances = false;
                if localities[&locality][&current_name].len() > 1 {
                    let f = |s: &&Stance| {
                        return if let Some(&ref indicator) = s.indicator.as_ref() {
                            indicator != &stance && !indicator.chars().all(char::is_numeric)
                        } else {
                            false
                        }
                    };
                    if localities[&locality][&current_name].iter().filter(f).count() != 0 {
                        include_original_stances = true;
                    }
                }
                // Initialise stop if it doesn't exist - don't overwrite what may already be there
                localities.get_mut(&locality).unwrap().entry(stop.clone()).or_insert_with(|| vec![]);
                // But regardless, merge all stances associated with the stop
                for original_stop in localities.get_mut(&locality).unwrap().get_mut(&current_name).unwrap() {
                    if include_original_stances {
                        original_stop.indicator = Some(format!("{} {}", original_stop.indicator.clone().unwrap_or_default(), stance))
                    } else {
                        original_stop.indicator = Some(stance.clone());
                    }
                }
                let mut new_locs = localities.get_mut(&locality).unwrap().get_mut(&current_name).unwrap().clone();
                localities.get_mut(&locality).unwrap().get_mut(&stop).unwrap().append(&mut new_locs);
                localities.get_mut(&locality).unwrap().remove(&current_name);
            }
        }
    }
    Ok(())
}

fn split_stop_name<'h>(stop: &'h str, stances_regex: &Regex, interchange_regex: &Regex, leeds_regex: &Regex, station_regex: &Regex) -> Option<(String, String, String)> {
    let pattern = stances_regex.captures(stop)
        .or_else(|| interchange_regex.captures(stop))
        .or_else(|| leeds_regex.captures(stop))
        .or_else(|| station_regex.captures(stop));
    if let Some(pattern) = pattern {
        let stop_stem = pattern.name("stop")?.as_str();
        let stance = pattern.name("stance")?.as_str();
        if stop_stem == "Number" || stop_stem.contains("H&R") || stop_stem.contains("H & R") || stop_stem.ends_with("No")
            || stop_stem.ends_with("No.") || stop_stem.ends_with("Unit") || stop_stem.ends_with("Heathrow Terminal") {
            return None;
        } else {
            return Some((stop.to_string(), stop_stem.to_string(), stance.to_string()));
        }
    } else {
        return None;
    }
}

fn write_groups(groupings: &Localities) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(BufWriter::new(File::create("localities.json")?), groupings)?;
    Ok(())
}

fn load_csv(path: &str) -> Result<DataFrame, Box<dyn Error>> {
    // needed due to inconsistency of 0/false
    let mut schema_overwrite = Schema::new();
    schema_overwrite.with_column(SmartString::from("LocalityCentre"), DataType::String);

    Ok(CsvReadOptions::default()
        .with_schema_overwrite(Some(SchemaRef::new(schema_overwrite)))
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?)
}