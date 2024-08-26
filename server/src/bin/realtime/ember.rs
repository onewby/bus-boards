use std::sync::Arc;
use prost::Message;
use tokio::sync::mpsc::Sender;
use tokio::time;
use BusBoardsServer::config::BBConfig;
use crate::GTFSResponder::EMBER;
use crate::{GTFSResponse, uw};
use crate::transit_realtime::{Alert, EntitySelector, FeedEntity, FeedMessage, TripDescriptor, TripUpdate, VehiclePosition};

pub async fn ember_listener(tx: Sender<GTFSResponse>, _: Arc<BBConfig>) {
    loop {
        let mut gtfs_rt: FeedMessage = FeedMessage::default();
        // Fetch Ember GTFS data
        if let Ok(result) = reqwest::get("https://api.ember.to/v1/gtfs/realtime/").await {
            if let Ok(bytes) = result.bytes().await {
                if let Ok(gtfs_rt2) = FeedMessage::decode(std::io::Cursor::new(bytes)) {
                    gtfs_rt = gtfs_rt2
                }
            }
        }

        // Map Ember GTFS -> local by adding the designated prefix
        let mut entities: Vec<FeedEntity> = gtfs_rt.entity.iter().cloned().map(|old_entity| {
            return FeedEntity {
                vehicle: old_entity.vehicle.map(|v| {
                    VehiclePosition {
                        trip: v.trip.map(|trip| {
                            TripDescriptor {
                                trip_id: trip.trip_id.map(|tid| "E".to_string() + tid.as_str()),
                                ..trip
                            }
                        }), ..v
                    }
                }),
                trip_update: old_entity.trip_update.map(|tu| {
                    TripUpdate {
                        trip: TripDescriptor {
                            trip_id: tu.trip.trip_id.map(|tid| "E".to_string() + tid.as_str()),
                            ..tu.trip
                        }, ..tu
                    }
                }),
                alert: old_entity.alert.map(|alert| {
                    Alert {
                        informed_entity: alert.informed_entity.iter().cloned().map(|e| {
                            EntitySelector {
                                trip: e.trip.map(|trip| {
                                    TripDescriptor {
                                        trip_id: trip.trip_id.map(|tid| "E".to_string() + tid.as_str()),
                                        ..trip
                                    }
                                }), ..e
                            }
                        }).collect(), ..alert
                    }
                }),
                ..old_entity
            };
        }).collect();

        // Separate alerts from vehicle data
        let alerts: Vec<Alert> = entities.iter().filter_map(|e| e.alert.clone()).collect();

        // Partition into vehicle data, trip updates
        let (tus, mut vehicles): (Vec<FeedEntity>, Vec<FeedEntity>) = entities.iter_mut().map(|e| e.clone()).partition(|e| e.trip_update.is_some() && e.vehicle.is_none());
        // Combine vehicle data with trip updates for the same trip
        vehicles.iter_mut().filter(|e| e.trip_update.is_none()).for_each(|trip| {
            if let Some(vehicleless) = tus.iter().find(|vehicleless| uw!(trip.vehicle.clone()?.trip?.trip_id) == uw!(vehicleless.trip_update.clone()?.trip.trip_id)) {
                trip.trip_update = vehicleless.trip_update.clone();
            }
        });

        // Send to main feed
        tx.send((EMBER, vehicles, alerts)).await.unwrap_or_else(|err| eprintln!("{}", err));

        // Wait until next loop
        time::sleep(time::Duration::from_secs(60)).await
    }
}