pub mod interop;
pub mod pipeline;

pub use pipeline::{CapturePipeline, CapturedFrame};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    #[test]
    fn test_capture_pipeline_frames() {
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
            let _ = windows::Win32::System::WinRT::RoInitialize(
                windows::Win32::System::WinRT::RO_INIT_MULTITHREADED,
            );
        }
        let (tx, rx) = crossbeam_channel::bounded(60);
        let is_capturing = Arc::new(AtomicBool::new(true));
        match CapturePipeline::new(None, tx, is_capturing.clone()) {
            Ok(mut p) => {
                println!("Pipeline created successfully: {}x{}", p.width, p.height);
                let start = std::time::Instant::now();
                let mut count = 0;
                while start.elapsed().as_millis() < 1000 {
                    if let Ok(_) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        count += 1;
                    }
                }
                println!("TOTAL CAPTURED FRAMES IN 1 SEC: {}", count);
                p.stop();
            }
            Err(e) => {
                println!("CapturePipeline::new error: {:?}", e);
            }
        }
    }
}