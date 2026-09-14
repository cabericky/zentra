pub mod audio;
pub mod compositor;
pub mod decoder;
pub mod device;
pub mod sink_writer;
pub mod timeline;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use crate::storage::Storage;
use audio::pump_segment_audio;
use compositor::ZoomCompositor;
use decoder::open_segment_reader;
use device::{create_export_d3d_device, create_export_textures};
use sink_writer::{create_export_sink_writer, write_video_frame};
use timeline::KeyframeTimeline;

#[derive(Debug, Clone, Serialize)]
pub struct ExportProgress {
    pub session_id: String,
    pub percent: f32,
    pub current_frame: u64,
    pub total_frames: u64,
    pub fps: f32,
    pub status: String,
}

struct MfComGuard {
    _private: (),
}

impl MfComGuard {
    fn new() -> Self {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = MFStartup(MF_VERSION, MFSTARTUP_FULL);
        }
        Self { _private: () }
    }
}

impl Drop for MfComGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
            CoUninitialize();
        }
    }
}

pub struct SessionExporter;

impl SessionExporter {
    /// Exports a completed recording session into a single MP4 video with smooth cubic
    /// zoom-on-click compositing.
    pub fn export_session(
        app_handle: AppHandle,
        session_id: String,
        storage: Arc<Storage>,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<PathBuf, String> {
        let _mf_com_guard = MfComGuard::new();

        let session = storage
            .get_session(&session_id)
            .map_err(|e| format!("Failed to load session {}: {:?}", session_id, e))?;

        let segments = storage
            .get_segments(&session_id)
            .map_err(|e| format!("Failed to load segments: {:?}", e))?;

        if segments.is_empty() {
            return Err("No recorded video segments found for this session.".to_string());
        }

        let clicks = storage
            .get_clicks(&session_id)
            .map_err(|e| format!("Failed to load clicks: {:?}", e))?;

        let pan_samples = storage
            .get_pan_samples(&session_id)
            .unwrap_or_default();

        log::info!(
            "Starting export for session {} with {} segments, {} clicks, and {} pan samples",
            session_id,
            segments.len(),
            clicks.len(),
            pan_samples.len(),
        );

        let output_dir = PathBuf::from(&session.output_dir);
        let export_file_name = if let Some(first_seg) = segments.first() {
            let seg_path = Path::new(&first_seg.file_path);
            if let Some(stem) = seg_path.file_stem().and_then(|s| s.to_str()) {
                if segments.len() > 1 {
                    let last_stem = Path::new(&segments.last().unwrap().file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    format!("{}_{}_zoom.mp4", stem, last_stem)
                } else {
                    format!("{}_zoom.mp4", stem)
                }
            } else {
                format!("segment_{:04}_zoom.mp4", first_seg.segment_index)
            }
        } else {
            format!("zentra_export_{}.mp4", session_id)
        };
        let export_path = output_dir.join(&export_file_name);

        let width = session.width;
        let height = session.height;
        let fps = session.fps.max(30);

        // Build Keyframe timeline from click log and continuous pan samples
        let timeline = KeyframeTimeline::build_with_pan_samples(&clicks, &pan_samples, width, height);

        // Calculate total session duration and expected CFR output frame count
        let total_session_duration = if session.end_qpc > session.start_qpc && session.qpc_frequency > 0 {
            (session.end_qpc - session.start_qpc) as f64 / session.qpc_frequency as f64
        } else {
            0.0
        };

        let total_frames: u64 = if total_session_duration > 0.0 {
            (total_session_duration * fps as f64).ceil() as u64
        } else {
            segments.iter().map(|s| s.frame_count.max(1)).sum()
        };

        // 1. Create D3D11 Device & Textures for export compositing via modular device setup
        let (d3d_device, d3d_context, d3d_manager) = create_export_d3d_device()?;
        let (src_texture, dst_texture) = create_export_textures(&d3d_device, width, height)?;

        // Initialize Zoom Compositor
        let mut compositor = ZoomCompositor::new(&d3d_device, width, height)
            .map_err(|e| format!("Failed to create ZoomCompositor: {:?}", e))?;

        // 2. Setup Media Foundation Sink Writer via modular sink_writer
        let bitrate = 16_000_000u32; // 16 Mbps high quality master export
        let (sink_writer, export_stream_idx, export_audio_stream_idx) = create_export_sink_writer(
            &export_path,
            width,
            height,
            fps,
            bitrate,
            d3d_manager.as_ref(),
        )?;

        let output_interval = 1.0 / (fps as f64);
        let hns_frame_duration = (10_000_000.0 / fps as f64) as i64;
        let mut overall_frame_index = 0u64;
        let mut current_output_time = 0.0f64;
        let mut has_loaded_frame = false;
        let export_start_time = Instant::now();

        macro_rules! emit_cfr_frame {
            ($time:expr, $idx:expr) => {{
                let transform = timeline.evaluate($time);
                let _ = compositor.composite(&src_texture, &dst_texture, &transform);

                let sample_time = $idx as i64 * hns_frame_duration;
                let _ = write_video_frame(
                    &sink_writer,
                    export_stream_idx,
                    &dst_texture,
                    sample_time,
                    hns_frame_duration,
                    width,
                    height,
                );

                if $idx % 15 == 0 {
                    let elapsed = export_start_time.elapsed().as_secs_f32().max(0.001);
                    let current_fps = ($idx as f32) / elapsed;
                    let percent = (($idx as f32) / (total_frames.max(1) as f32) * 100.0).min(99.0);

                    let prog = ExportProgress {
                        session_id: session_id.clone(),
                        percent,
                        current_frame: $idx,
                        total_frames,
                        fps: current_fps,
                        status: "rendering".to_string(),
                    };
                    let _ = app_handle.emit("zentra://export-progress", prog);
                }
            }};
        }

        // 3. Process each video segment sequentially
        for segment in &segments {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err("Export cancelled by user.".to_string());
            }

            let seg_path = Path::new(&segment.file_path);
            if !seg_path.exists() {
                log::warn!("Segment file does not exist: {}", segment.file_path);
                continue;
            }

            // Open segment reader via modular decoder
            let (reader, has_segment_audio) = match open_segment_reader(seg_path, export_audio_stream_idx.is_some()) {
                Ok(res) => res,
                Err(e) => {
                    log::error!("Failed to open segment reader: {}", e);
                    continue;
                }
            };

            let seg_start_seconds = if session.qpc_frequency > 0 {
                (segment.start_qpc - session.start_qpc) as f64 / session.qpc_frequency as f64
            } else {
                0.0
            };

            let mut last_audio_time_sec = 0.0f64;

            loop {
                if cancel_flag.load(Ordering::Relaxed) {
                    return Err("Export cancelled by user.".to_string());
                }

                let mut actual_stream_idx = 0u32;
                let mut stream_flags = 0u32;
                let mut timestamp = 0i64;
                let mut sample: Option<IMFSample> = None;

                let hr = unsafe {
                    reader.ReadSample(
                        MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                        0,
                        Some(&mut actual_stream_idx),
                        Some(&mut stream_flags),
                        Some(&mut timestamp),
                        Some(&mut sample),
                    )
                };

                if hr.is_err() || (stream_flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32) != 0 {
                    break;
                }

                if let Some(src_sample) = sample {
                    unsafe {
                        if let Ok(buf) = src_sample.GetBufferByIndex(0) {
                            let mut ptr: *mut u8 = std::ptr::null_mut();
                            let mut max_len = 0u32;
                            let mut current_len = 0u32;

                            if buf.Lock(&mut ptr, Some(&mut max_len), Some(&mut current_len)).is_ok() {
                                let input_frame_time = seg_start_seconds + (timestamp as f64 / 10_000_000.0);

                                // Pump and mux audio samples up to input_frame_time via modular audio handler
                                if has_segment_audio {
                                    if let Some(audio_idx) = export_audio_stream_idx {
                                        pump_segment_audio(
                                            &reader,
                                            &sink_writer,
                                            audio_idx,
                                            seg_start_seconds,
                                            Some(input_frame_time),
                                            &mut last_audio_time_sec,
                                        );
                                    }
                                }

                                // Fill CFR output frames up to this new frame's timestamp
                                if has_loaded_frame {
                                    while current_output_time < input_frame_time && (total_session_duration == 0.0 || current_output_time < total_session_duration) {
                                        if cancel_flag.load(Ordering::Relaxed) {
                                            let _ = buf.Unlock();
                                            return Err("Export cancelled by user.".to_string());
                                        }
                                        emit_cfr_frame!(current_output_time, overall_frame_index);
                                        overall_frame_index += 1;
                                        current_output_time = overall_frame_index as f64 * output_interval;
                                    }
                                }

                                // Upload new raw RGB32 frame buffer to source D3D11 texture
                                let row_pitch = width * 4;
                                d3d_context.UpdateSubresource(
                                    &src_texture,
                                    0,
                                    None,
                                    ptr as *const std::ffi::c_void,
                                    row_pitch,
                                    0,
                                );
                                let _ = buf.Unlock();

                                // If this was the first frame, backfill from t=0.0 up to input_frame_time
                                if !has_loaded_frame {
                                    has_loaded_frame = true;
                                    while current_output_time < input_frame_time && (total_session_duration == 0.0 || current_output_time < total_session_duration) {
                                        if cancel_flag.load(Ordering::Relaxed) {
                                            return Err("Export cancelled by user.".to_string());
                                        }
                                        emit_cfr_frame!(current_output_time, overall_frame_index);
                                        overall_frame_index += 1;
                                        current_output_time = overall_frame_index as f64 * output_interval;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Drain remaining audio samples from this segment to EOF
            if has_segment_audio {
                if let Some(audio_idx) = export_audio_stream_idx {
                    pump_segment_audio(
                        &reader,
                        &sink_writer,
                        audio_idx,
                        seg_start_seconds,
                        None,
                        &mut last_audio_time_sec,
                    );
                }
            }
        }

        // 4. Flush remaining output frames up to total_session_duration holding the final screen frame
        if has_loaded_frame {
            if total_session_duration > 0.0 {
                while current_output_time < total_session_duration {
                    if cancel_flag.load(Ordering::Relaxed) {
                        return Err("Export cancelled by user.".to_string());
                    }
                    emit_cfr_frame!(current_output_time, overall_frame_index);
                    overall_frame_index += 1;
                    current_output_time = overall_frame_index as f64 * output_interval;
                }
            } else if overall_frame_index == 0 {
                emit_cfr_frame!(0.0, 0);
                overall_frame_index += 1;
            }
        }

        // 5. Finalize export sink writer
        unsafe {
            let _ = sink_writer.Finalize();
        }

        // Update database export status
        let _ = storage.update_export_status(&session_id, "exported");

        // Emit final 100% progress
        let final_prog = ExportProgress {
            session_id: session_id.clone(),
            percent: 100.0,
            current_frame: overall_frame_index,
            total_frames: overall_frame_index,
            fps: (overall_frame_index as f32) / export_start_time.elapsed().as_secs_f32().max(0.001),
            status: "completed".to_string(),
        };
        let _ = app_handle.emit("zentra://export-progress", final_prog);

        log::info!(
            "Export completed successfully: {} ({} frames)",
            export_path.display(),
            overall_frame_index
        );

        Ok(export_path)
    }
}