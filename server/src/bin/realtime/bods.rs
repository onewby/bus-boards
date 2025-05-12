use std::io::Read;
use std::sync::Arc;
use geo_types::Point;
use itertools::Itertools;
use log::error;
use prost::Message;
use tokio::sync::mpsc::Sender;
use tokio::time;
use zip::ZipArchive;
use BusBoardsServer::config::BBConfig;
use crate::db::{get_bods_trip, get_line_segments, DBPool};
use crate::GTFSResponder::BODS;
use crate::{uw, GTFSResponse};
use crate::transit_realtime::{FeedEntity, FeedMessage, VehiclePosition};
use crate::util::{f64_cmp, get_geo_linepoint_distance};

pub async fn bods_listener(tx: Sender<GTFSResponse>, _: Arc<BBConfig>, db: Arc<DBPool>) {
    loop {
        let mut bods: FeedMessage = FeedMessage::default();
        // Download + decode BODS data
        if let Ok(result) = reqwest::get("https://data.bus-data.dft.gov.uk/avl/download/gtfsrt").await {
            if let Ok(bytes) = result.bytes().await {
                if let Ok(mut archive) = ZipArchive::new(std::io::Cursor::new(bytes)) {
                    if let Ok(zip_file) = archive.by_name("gtfsrt.bin") {
                        if let Ok(bods2) = FeedMessage::decode(std::io::Cursor::new(zip_file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>())) {
                            bods = bods2
                        }
                    }
                }
            }
        }
        // Include vehicles with an active journey
        if bods.header.timestamp.is_some() {
            let filtered_entities: Vec<FeedEntity> = bods.entity.iter()
                .filter(|e| uw!(e.vehicle.as_ref()?.trip.as_ref()?.trip_id.as_ref()).map_or(false, |tid| !tid.is_empty()))
                .map(|e| {
                    if let Some(pos) = &e.vehicle.as_ref().unwrap().position {
                        if e.vehicle.as_ref().unwrap().current_stop_sequence.is_none() {
                            let trip = uw!(e.vehicle.as_ref()?.trip.as_ref()).unwrap();
                            let loc = Point::new(pos.longitude as f64, pos.latitude as f64);
                            let info = get_bods_trip(&db, trip.trip_id());
                            if let Some(info) = info {
                                let points = get_line_segments(&db, info.route_id);
                                let route = &info.trip_route;
                                let seqs = &info.trip_seqs;
                                let segments: Vec<geo_types::Line<f64>> = (0..route.len()-1).map(|i| {
                                    geo_types::Line::new(points.get(&route[i]).copied().unwrap_or_default(), points.get(&route[i+1]).copied().unwrap_or_default())
                                }).collect();
                                let closest_segment = segments.iter().map(|s| get_geo_linepoint_distance(s, &loc))
                                    .position_min_by(f64_cmp).unwrap_or(0);
                                return FeedEntity {
                                    vehicle: Some(VehiclePosition {
                                        current_stop_sequence: Some((seqs[closest_segment] + 1).min(*seqs.last().unwrap())),
                                        stop_id: Some(route[closest_segment].to_string()),
                                        ..e.vehicle.as_ref().unwrap().clone()
                                    }),
                                    ..e.clone()
                                }
                            }
                        }
                        e.clone()
                    } else {
                        e.clone()
                    }
                }).collect();
            // Send to main feed
            tx.send((BODS, filtered_entities, vec![])).await.unwrap_or_else(|err| error!("{}", err));
        }
        // Wait for next loop
        time::sleep(time::Duration::from_secs(60)).await
    }
}