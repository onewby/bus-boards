use std::io::Read;
use std::sync::Arc;
use prost::Message;
use tokio::sync::mpsc::Sender;
use tokio::time;
use zip::ZipArchive;
use BusBoardsServer::config::BBConfig;
use crate::GTFSResponder::BODS;
use crate::GTFSResponse;
use crate::transit_realtime::{FeedEntity, FeedMessage};

pub async fn bods_listener(tx: Sender<GTFSResponse>, _: Arc<BBConfig>) {
    loop {
        let mut bods: FeedMessage = FeedMessage::default();
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
        if bods.header.timestamp.is_some() {
            let filtered_entities: Vec<FeedEntity> = bods.entity.iter()
                .filter(|e| {
                    if let Some(v) = &e.vehicle {
                        if let Some(t) = &v.trip {
                            if let Some(tid) = &t.trip_id {
                                return !tid.is_empty();
                            }
                        }
                    }
                    false
                })
                .cloned().collect();
            tx.send((BODS, filtered_entities, vec![])).await.unwrap_or_else(|err| eprintln!("{}", err));
        }
        time::sleep(time::Duration::from_secs(60)).await
    }
}