use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::Receiver;
use parking_lot::RwLock;

use crate::storage::{ClickRecord, Storage};
use super::types::{ClickEvent, RawClickEvent};

pub fn spawn_writer_thread(
    session_id: String,
    qpc_start: i64,
    qpc_frequency: i64,
    storage: Arc<Storage>,
    offset_x: i32,
    offset_y: i32,
    raw_receiver: Receiver<RawClickEvent>,
    is_running: Arc<AtomicBool>,
    recent_clicks: Arc<RwLock<Vec<ClickEvent>>>,
    magnifier_callback: Option<Arc<dyn Fn(&str, i32, i32) + Send + Sync + 'static>>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("zentra-mouse-writer".to_string())
        .spawn(move || {
            let mut pending_batch: Vec<ClickRecord> = Vec::with_capacity(64);

            while is_running.load(Ordering::Relaxed) || !raw_receiver.is_empty() {
                // Drain up to 64 events or wait up to 50ms
                match raw_receiver.recv_timeout(Duration::from_millis(50)) {
                    Ok(raw) => {
                        let rel_x = raw.point.x - offset_x;
                        let rel_y = raw.point.y - offset_y;

                        if let Some(ref cb) = magnifier_callback {
                            cb(raw.button, rel_x, rel_y);
                        }

                        let time_seconds = if qpc_frequency > 0 {
                            (raw.qpc - qpc_start) as f64 / qpc_frequency as f64
                        } else {
                            0.0
                        };

                        let event = ClickEvent {
                            session_id: session_id.clone(),
                            qpc_timestamp: raw.qpc,
                            time_seconds,
                            x: rel_x,
                            y: rel_y,
                            button: raw.button.to_string(),
                        };

                        let record = ClickRecord {
                            id: None,
                            session_id: session_id.clone(),
                            qpc_timestamp: raw.qpc,
                            time_seconds,
                            x: rel_x,
                            y: rel_y,
                            button: raw.button.to_string(),
                        };

                        // Update in-memory ring for debug overlay / export preview
                        {
                            let mut recent = recent_clicks.write();
                            recent.push(event);
                            if recent.len() > 5000 {
                                let drain_count = recent.len() - 3000;
                                recent.drain(0..drain_count);
                            }
                        }

                        pending_batch.push(record);
                    }
                    Err(_) => {}
                }

                // Flush batch if threshold reached or idle
                if pending_batch.len() >= 32 || (!pending_batch.is_empty() && raw_receiver.is_empty()) {
                    if let Err(e) = storage.insert_clicks_batch(&pending_batch) {
                        log::error!("Error writing clicks to SQLite: {:?}", e);
                    }
                    pending_batch.clear();
                }
            }

            // Final flush
            if !pending_batch.is_empty() {
                let _ = storage.insert_clicks_batch(&pending_batch);
            }
            log::info!("Mouse writer thread finished.");
        })
        .expect("failed to spawn mouse writer thread")
}