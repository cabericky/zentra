pub mod animation;
pub mod ffi;

pub use animation::{MagnifierCmd, Transform};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use crossbeam_channel::{unbounded, Receiver, Sender};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

use crate::storage::PanSampleRecord;

/// Controller managing the native Windows Magnification API display state and camera motion trajectory.
/// Runs a dedicated 60 FPS animation thread to smoothly interpolate screen transforms.
pub struct LiveMagnifierController {
    sender: Sender<MagnifierCmd>,
    worker_handle: Option<JoinHandle<()>>,
    is_active: Arc<AtomicBool>,
    shared_transform: Arc<parking_lot::RwLock<crate::export::timeline::TransformState>>,
}

impl LiveMagnifierController {
    /// Resets the physical display to 1.0x full screen using Magnification.dll.
    pub fn reset_display() {
        ffi::MagnificationApi::reset_display();
    }

    /// Starts the live screen magnifier and camera trajectory recorder.
    /// Screen begins at 1.0x (normal view) and listens for zoom commands.
    pub fn start(
        screen_width: u32,
        screen_height: u32,
        offset_x: i32,
        offset_y: i32,
        session_id: String,
        qpc_start: i64,
        qpc_frequency: i64,
        storage: Arc<crate::storage::Storage>,
        enable_display: bool,
    ) -> Result<Self, String> {
        let (sender, receiver): (Sender<MagnifierCmd>, Receiver<MagnifierCmd>) = unbounded();
        let is_active = Arc::new(AtomicBool::new(true));
        let is_active_clone = Arc::clone(&is_active);

        let sw = if screen_width > 0 {
            screen_width as f32
        } else {
            unsafe { GetSystemMetrics(SM_CXSCREEN) as f32 }
        };

        let sh = if screen_height > 0 {
            screen_height as f32
        } else {
            unsafe { GetSystemMetrics(SM_CYSCREEN) as f32 }
        };

        let shared_transform = Arc::new(parking_lot::RwLock::new(crate::export::timeline::TransformState {
            scale: 1.0,
            center_x: sw / 2.0,
            center_y: sh / 2.0,
        }));
        let shared_transform_clone = Arc::clone(&shared_transform);

        let worker_handle = thread::Builder::new()
            .name("zentra-live-magnifier".to_string())
            .spawn(move || {
                unsafe {
                    let _ = ffi::timeBeginPeriod(1);
                }

                let mag_api = if enable_display {
                    ffi::MagnificationApi::load()
                } else {
                    None
                };

                log::info!("Live Screen Magnifier initialized for screen {}x{} (display: {})", sw, sh, enable_display);

                let peak_mag = 1.8f32;
                let transition_duration = Duration::from_millis(300);

                let mut current = Transform::default();
                let mut from = Transform::default();
                let mut target = Transform::default();
                let mut transition_start = Instant::now();
                let mut in_transition = false;
                let mut is_zoomed = false;
                let mut was_zoomed = false;

                let mut pending_pan_samples: Vec<PanSampleRecord> = Vec::with_capacity(64);

                while is_active_clone.load(Ordering::Relaxed) {
                    // Check for incoming commands (drain all latest)
                    let mut received_cmd = None;
                    while let Ok(cmd) = receiver.try_recv() {
                        received_cmd = Some(cmd);
                    }

                    if let Some(cmd) = received_cmd {
                        match cmd {
                            MagnifierCmd::ZoomIn { x, y } => {
                                let (target_x, target_y) = animation::calculate_target_offset(x as f32, y as f32, sw, sh, peak_mag);
                                from = current;
                                target = Transform {
                                    mag: peak_mag,
                                    offset_x: target_x,
                                    offset_y: target_y,
                                };
                                transition_start = Instant::now();
                                in_transition = true;
                                is_zoomed = true;
                                log::debug!("Magnifier ZoomIn target: ({}, {}) at 1.8x", target_x, target_y);
                            }
                            MagnifierCmd::ZoomOut => {
                                from = current;
                                target = Transform::default();
                                transition_start = Instant::now();
                                in_transition = true;
                                is_zoomed = false;
                                log::debug!("Magnifier ZoomOut target: 1.0x");
                            }
                            MagnifierCmd::Stop => {
                                break;
                            }
                        }
                    }

                    // Animate interpolation if in transition
                    if in_transition {
                        let elapsed = transition_start.elapsed();
                        if elapsed >= transition_duration {
                            current = target;
                            in_transition = false;
                        } else {
                            let progress = elapsed.as_secs_f32() / transition_duration.as_secs_f32();
                            current = animation::interpolate_transform(from, target, progress);
                        }

                        if let Some(ref api) = mag_api {
                            api.set_transform(
                                current.mag,
                                offset_x + current.offset_x.round() as i32,
                                offset_y + current.offset_y.round() as i32,
                            );
                        }
                    } else if is_zoomed && current.mag > 1.05 {
                        // Screen is already zoomed in: freely drag screen following the mouse cursor with zero delay
                        let mut pt = POINT::default();
                        unsafe {
                            let _ = GetCursorPos(&mut pt);
                        }

                        let cur_x = (pt.x - offset_x) as f32;
                        let cur_y = (pt.y - offset_y) as f32;

                        let (cursor_target_x, cursor_target_y) = animation::calculate_target_offset(cur_x, cur_y, sw, sh, current.mag);

                        // Direct zero-delay tracking
                        current.offset_x = cursor_target_x;
                        current.offset_y = cursor_target_y;

                        if let Some(ref api) = mag_api {
                            api.set_transform(
                                current.mag,
                                offset_x + current.offset_x.round() as i32,
                                offset_y + current.offset_y.round() as i32,
                            );
                        }
                    }

                    // Update real-time shared transform state for pipeline sync
                    let center_x = current.offset_x + (sw / current.mag) * 0.5;
                    let center_y = current.offset_y + (sh / current.mag) * 0.5;
                    {
                        let mut st = shared_transform_clone.write();
                        st.scale = current.mag;
                        st.center_x = center_x;
                        st.center_y = center_y;
                    }

                    // Record pan/zoom trajectory sample to SQLite during active zoom
                    let is_active_zoom = current.mag > 1.001;
                    if is_active_zoom {
                        let mut qpc_now = 0i64;
                        unsafe {
                            let _ = windows::Win32::System::Performance::QueryPerformanceCounter(&mut qpc_now);
                        }
                        let time_seconds = if qpc_frequency > 0 {
                            (qpc_now - qpc_start) as f64 / qpc_frequency as f64
                        } else {
                            0.0
                        };

                        pending_pan_samples.push(PanSampleRecord {
                            id: None,
                            session_id: session_id.clone(),
                            qpc_timestamp: qpc_now,
                            time_seconds,
                            scale: current.mag,
                            center_x,
                            center_y,
                        });

                        if pending_pan_samples.len() >= 30 {
                            let _ = storage.insert_pan_samples_batch(&pending_pan_samples);
                            pending_pan_samples.clear();
                        }
                    }

                    // Flush on zoom-out complete
                    if was_zoomed && !is_active_zoom && !pending_pan_samples.is_empty() {
                        let _ = storage.insert_pan_samples_batch(&pending_pan_samples);
                        pending_pan_samples.clear();
                    }
                    was_zoomed = is_active_zoom;

                    thread::sleep(Duration::from_millis(16)); // ~60 FPS
                }

                // Flush any remaining pan samples
                if !pending_pan_samples.is_empty() {
                    let _ = storage.insert_pan_samples_batch(&pending_pan_samples);
                    pending_pan_samples.clear();
                }

                // Reset to 1.0x full screen on stop
                if let Some(api) = mag_api {
                    api.shutdown();
                }

                unsafe {
                    let _ = ffi::timeEndPeriod(1);
                }
                log::info!("Live Screen Magnifier safely terminated and reset to 1.0x.");
            })
            .map_err(|e| format!("Failed to spawn magnifier animation thread: {:?}", e))?;

        Ok(Self {
            sender,
            worker_handle: Some(worker_handle),
            is_active,
            shared_transform,
        })
    }

    /// Provides access to the real-time shared transform state.
    pub fn shared_transform(&self) -> Arc<parking_lot::RwLock<crate::export::timeline::TransformState>> {
        Arc::clone(&self.shared_transform)
    }

    /// Retrieves the current snapshot of the transform state.
    pub fn current_transform(&self) -> crate::export::timeline::TransformState {
        *self.shared_transform.read()
    }

    /// Triggers zoom in to cursor location (Backtick key).
    pub fn zoom_in(&self, x: i32, y: i32) {
        let _ = self.sender.send(MagnifierCmd::ZoomIn { x, y });
    }

    /// Triggers zoom out back to 1.0x full screen (Esc key).
    pub fn zoom_out(&self) {
        let _ = self.sender.send(MagnifierCmd::ZoomOut);
    }

    /// Stops and resets the magnifier via shared reference.
    pub fn stop_and_reset(&self) {
        self.is_active.store(false, Ordering::SeqCst);
        let _ = self.sender.send(MagnifierCmd::Stop);
    }

    /// Safely stops the magnifier and joins the thread.
    pub fn stop(&mut self) {
        self.is_active.store(false, Ordering::SeqCst);
        let _ = self.sender.send(MagnifierCmd::Stop);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for LiveMagnifierController {
    fn drop(&mut self) {
        self.stop();
    }
}