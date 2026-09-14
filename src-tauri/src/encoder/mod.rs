pub mod rolling;
pub mod scanner;
pub mod segment_writer;

pub use rolling::RollingEncoder;
pub use scanner::get_next_segment_index;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use crate::clock::QpcClock;
    use crate::storage::Storage;

    #[test]
    fn test_encoder_recording() {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
            let _ = windows::Win32::System::WinRT::RoInitialize(
                windows::Win32::System::WinRT::RO_INIT_MULTITHREADED,
            );
        }

        let test_dir = std::env::temp_dir().join("zentra_encoder_test");
        let _ = std::fs::create_dir_all(&test_dir);
        let db_path = test_dir.join("test.db");
        let storage = Arc::new(Storage::new(&db_path).unwrap());

        let (tx, rx) = crossbeam_channel::bounded(60);
        let is_capturing = Arc::new(AtomicBool::new(true));

        let mut capture = match crate::capture::CapturePipeline::new(None, tx, is_capturing.clone()) {
            Ok(c) => c,
            Err(e) => {
                println!("CapturePipeline failed: {:?}", e);
                return;
            }
        };

        let (_audio_tx, audio_rx) = crossbeam_channel::unbounded();

        let clock = QpcClock::new();
        let qpc_start = clock.now();
        let session_id = format!("test_enc_{}", chrono::Utc::now().timestamp_millis());

        let session_record = crate::storage::SessionRecord {
            id: session_id.clone(),
            start_qpc: qpc_start,
            end_qpc: 0,
            qpc_frequency: clock.frequency,
            width: capture.width,
            height: capture.height,
            fps: 60,
            output_dir: test_dir.to_string_lossy().to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            export_status: "recorded".to_string(),
        };
        storage.insert_session(&session_record).unwrap();

        let mut encoder = RollingEncoder::start(
            session_id.clone(),
            test_dir.clone(),
            1,
            capture.width,
            capture.height,
            60,
            clock,
            qpc_start,
            300.0,
            rx,
            audio_rx,
            Arc::clone(&storage),
            false,
            false,
            capture.d3d11_device.clone(),
            Arc::new(|| Vec::new()),
            None,
        )
        .expect("Failed to start encoder");

        println!("Recording for 1.5 seconds...");
        std::thread::sleep(std::time::Duration::from_millis(1500));

        capture.stop();
        encoder.stop();

        let segments = storage.get_segments(&session_id).unwrap();
        println!("TEST RESULT SEGMENTS: {:?}", segments);
        let mp4_file = test_dir.join("segment_0000.mp4");
        println!("MP4 FILE EXISTS: {}, SIZE: {:?}", mp4_file.exists(), std::fs::metadata(&mp4_file).map(|m| m.len()));
    }
}