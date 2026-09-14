use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use crossbeam_channel::Receiver;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use super::segment_writer::SegmentWriter;
use crate::audio::AudioChunk;
use crate::capture::CapturedFrame;
use crate::clock::QpcClock;
use crate::debug_overlay::DebugOverlay;
use crate::mouse_hook::ClickEvent;
use crate::storage::{SegmentRecord, Storage};

pub struct RollingEncoder {
    worker_handle: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
}

impl RollingEncoder {
    /// Spawns the encoding thread that consumes captured GPU frames,
    /// applies debug overlay if enabled, and writes rolling MP4 segments via Media Foundation.
    pub fn start(
        session_id: String,
        output_dir: PathBuf,
        start_segment_index: u32,
        width: u32,
        height: u32,
        fps: u32,
        qpc_clock: QpcClock,
        qpc_session_start: i64,
        segment_max_duration_seconds: f64,
        frame_receiver: Receiver<CapturedFrame>,
        audio_receiver: Receiver<AudioChunk>,
        storage: Arc<Storage>,
        debug_mode: bool,
        live_zoom: bool,
        d3d11_device: ID3D11Device,
        recent_clicks_provider: Arc<dyn Fn() -> Vec<ClickEvent> + Send + Sync + 'static>,
        shared_transform: Option<Arc<parking_lot::RwLock<crate::export::timeline::TransformState>>>,
    ) -> windows::core::Result<Self> {
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = Arc::clone(&is_running);

        let worker_handle = thread::Builder::new()
            .name("zentra-mf-encoder".to_string())
            .spawn(move || {
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                    let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
                }

                let mut debug_overlay = if debug_mode {
                    DebugOverlay::new(&d3d11_device, width, height).ok()
                } else {
                    None
                };

                let mut live_compositor = if live_zoom {
                    crate::export::compositor::ZoomCompositor::new(&d3d11_device, width, height).ok()
                } else {
                    None
                };

                let live_zoom_dest_texture = if live_zoom {
                    let texture_desc = D3D11_TEXTURE2D_DESC {
                        Width: width,
                        Height: height,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
                        CPUAccessFlags: 0,
                        MiscFlags: 0,
                    };
                    let mut tex = None;
                    unsafe {
                        let _ = d3d11_device.CreateTexture2D(&texture_desc, None, Some(&mut tex));
                    }
                    tex
                } else {
                    None
                };

                // Create DXGI Device Manager for hardware acceleration
                let mut reset_token = 0u32;
                let mut d3d_manager: Option<IMFDXGIDeviceManager> = None;
                unsafe {
                    if MFCreateDXGIDeviceManager(&mut reset_token, &mut d3d_manager).is_ok() {
                        if let Some(ref manager) = d3d_manager {
                            let _ = manager.ResetDevice(&d3d11_device, reset_token);
                        }
                    }
                }

                let mut segment_index = start_segment_index;
                let mut current_segment_frames = 0u64;
                let mut current_segment_start_qpc = qpc_session_start;
                let hns_frame_duration = (10_000_000.0 / fps as f64) as i64;
                let bitrate = 14_000_000u32; // 14 Mbps high-definition

                // Initialize segment 0
                let mut active_writer = SegmentWriter::open(
                    &output_dir,
                    segment_index,
                    width,
                    height,
                    fps,
                    bitrate,
                    d3d_manager.as_ref(),
                );
                let mut current_segment_audio_hns: i64 = 0;
                if let Some(ref writer) = active_writer {
                    let seg_record = SegmentRecord {
                        id: None,
                        session_id: session_id.clone(),
                        segment_index,
                        file_path: writer.file_path().to_string_lossy().to_string(),
                        start_qpc: current_segment_start_qpc,
                        end_qpc: 0,
                        frame_count: 0,
                    };
                    let _ = storage.insert_segment(&seg_record);
                }

                while is_running_clone.load(Ordering::Relaxed) || !frame_receiver.is_empty() || !audio_receiver.is_empty() {
                    // Drain pending audio chunks and write to active segment
                    while let Ok(chunk) = audio_receiver.try_recv() {
                        if let Some(ref writer) = active_writer {
                            let duration_hns = (chunk.sample_count as i64 * 10_000_000) / 48000;
                            let raw_hns = qpc_clock.to_hns(chunk.qpc_timestamp - current_segment_start_qpc).max(0);
                            let sample_time_hns = raw_hns.max(current_segment_audio_hns);
                            current_segment_audio_hns = sample_time_hns + duration_hns;
                            writer.write_audio_chunk(&chunk, sample_time_hns, duration_hns);
                        }
                    }

                    let frame = match frame_receiver.recv_timeout(std::time::Duration::from_millis(15)) {
                        Ok(f) => f,
                        Err(_) => continue,
                    };

                    // Check if we need to roll to a new segment
                    let segment_elapsed_secs = qpc_clock.delta_seconds(current_segment_start_qpc, frame.qpc_timestamp);
                    if segment_elapsed_secs >= segment_max_duration_seconds {
                        // Drain audio before finalizing current segment
                        while let Ok(chunk) = audio_receiver.try_recv() {
                            if let Some(ref writer) = active_writer {
                                let duration_hns = (chunk.sample_count as i64 * 10_000_000) / 48000;
                                let raw_hns = qpc_clock.to_hns(chunk.qpc_timestamp - current_segment_start_qpc).max(0);
                                let sample_time_hns = raw_hns.max(current_segment_audio_hns);
                                current_segment_audio_hns = sample_time_hns + duration_hns;
                                writer.write_audio_chunk(&chunk, sample_time_hns, duration_hns);
                            }
                        }

                        // Finalize current segment
                        if let Some(writer) = active_writer.take() {
                            let _ = writer.finalize();
                            let _ = storage.update_segment_end(
                                &session_id,
                                segment_index,
                                frame.qpc_timestamp,
                                current_segment_frames,
                            );
                            log::info!(
                                "Finalized segment {} ({} frames, {:.1}s)",
                                segment_index,
                                current_segment_frames,
                                segment_elapsed_secs
                            );
                        }

                        // Roll to next segment
                        segment_index += 1;
                        current_segment_frames = 0;
                        current_segment_start_qpc = frame.qpc_timestamp;
                        current_segment_audio_hns = 0;
                        active_writer = SegmentWriter::open(
                            &output_dir,
                            segment_index,
                            width,
                            height,
                            fps,
                            bitrate,
                            d3d_manager.as_ref(),
                        );

                        if let Some(ref writer) = active_writer {
                            let seg_record = SegmentRecord {
                                id: None,
                                session_id: session_id.clone(),
                                segment_index,
                                file_path: writer.file_path().to_string_lossy().to_string(),
                                start_qpc: current_segment_start_qpc,
                                end_qpc: 0,
                                frame_count: 0,
                            };
                            let _ = storage.insert_segment(&seg_record);
                        }
                    }

                    // Apply Debug Overlay if active
                    if let Some(ref mut overlay) = debug_overlay {
                        let clicks = recent_clicks_provider();
                        overlay.render(&frame.texture, frame.qpc_timestamp, &clicks, &qpc_clock);
                    }

                    // Apply Live Zoom on Click / Mouse Drag if active
                    let texture_to_write = if let (Some(ref mut comp), Some(ref dst_tex)) = (&mut live_compositor, &live_zoom_dest_texture) {
                        let transform = if let Some(ref st) = shared_transform {
                            *st.read()
                        } else {
                            let clicks = recent_clicks_provider();
                            if !clicks.is_empty() {
                                let click_records: Vec<crate::storage::ClickRecord> = clicks
                                    .iter()
                                    .map(|c| crate::storage::ClickRecord {
                                        id: None,
                                        session_id: session_id.clone(),
                                        qpc_timestamp: c.qpc_timestamp,
                                        time_seconds: c.time_seconds,
                                        x: c.x,
                                        y: c.y,
                                        button: c.button.clone(),
                                    })
                                    .collect();
                                let timeline = crate::export::timeline::KeyframeTimeline::build(&click_records, width, height);
                                let time_sec = qpc_clock.delta_seconds(qpc_session_start, frame.qpc_timestamp);
                                timeline.evaluate(time_sec)
                            } else {
                                crate::export::timeline::TransformState { scale: 1.0, center_x: width as f32 / 2.0, center_y: height as f32 / 2.0 }
                            }
                        };

                        if transform.scale > 1.001 {
                            let _ = comp.composite(&frame.texture, dst_tex, &transform);
                            dst_tex
                        } else {
                            &frame.texture
                        }
                    } else {
                        &frame.texture
                    };

                    // Write frame to Media Foundation sink writer
                    if let Some(ref writer) = active_writer {
                        let sample_time_hns = qpc_clock.to_hns(frame.qpc_timestamp - current_segment_start_qpc);
                        if writer.write_video_frame(
                            texture_to_write,
                            sample_time_hns,
                            hns_frame_duration,
                            width,
                            height,
                        ) {
                            current_segment_frames += 1;
                        } else {
                            log::warn!("Failed write_video_frame on segment {}", segment_index);
                        }
                    }
                }

                // Finalize active segment on recording stop
                while let Ok(chunk) = audio_receiver.try_recv() {
                    if let Some(ref writer) = active_writer {
                        let duration_hns = (chunk.sample_count as i64 * 10_000_000) / 48000;
                        let raw_hns = qpc_clock.to_hns(chunk.qpc_timestamp - current_segment_start_qpc).max(0);
                        let sample_time_hns = raw_hns.max(current_segment_audio_hns);
                        current_segment_audio_hns = sample_time_hns + duration_hns;
                        writer.write_audio_chunk(&chunk, sample_time_hns, duration_hns);
                    }
                }

                if let Some(writer) = active_writer.take() {
                    let _ = writer.finalize();
                    let mut qpc_end = 0i64;
                    unsafe {
                        let _ = windows::Win32::System::Performance::QueryPerformanceCounter(&mut qpc_end);
                    }
                    let _ = storage.update_segment_end(
                        &session_id,
                        segment_index,
                        qpc_end,
                        current_segment_frames,
                    );
                    log::info!("Cleanly finalized final segment {}", segment_index);
                }

                unsafe {
                    let _ = MFShutdown();
                    CoUninitialize();
                }
                log::info!("Encoder thread stopped.");
            })
            .expect("failed to spawn encoder thread");

        Ok(Self {
            worker_handle: Some(worker_handle),
            is_running,
        })
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RollingEncoder {
    fn drop(&mut self) {
        self.stop();
    }
}