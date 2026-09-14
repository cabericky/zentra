use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use crossbeam_channel::bounded;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::capture::CapturePipeline;
use crate::clock::QpcClock;
use crate::encoder::{self, RollingEncoder};
use crate::live_magnifier::LiveMagnifierController;
use crate::monitors;
use crate::mouse_hook::MouseHookController;
use crate::state::{ActiveSession, AppState, RecordingStatus};
use crate::storage::{SessionRecord, SessionSummary};
use crate::window_utils::{apply_capture_exclusion, is_widget_on_screen, position_widget_window};

#[tauri::command]
pub fn reset_screen_magnifier() {
    LiveMagnifierController::reset_display();
}

#[tauri::command]
pub fn get_recording_status(state: State<'_, AppState>) -> RecordingStatus {
    let lock = state.active_session.lock();
    if let Some(ref session) = *lock {
        let clock = QpcClock::new();
        let elapsed = clock.delta_seconds(session.start_qpc, clock.now());
        let clicks = session.mouse_hook.get_recent_clicks().len();
        RecordingStatus {
            is_recording: true,
            session_id: Some(session.session_id.clone()),
            elapsed_seconds: elapsed,
            click_count: clicks,
            segment_count: 1,
            debug_mode: session.debug_mode,
            live_zoom: session.live_zoom,
            live_magnifier: session.live_magnifier.is_some(),
            record_system_audio: session.record_system_audio,
            record_mic: session.record_mic,
        }
    } else {
        RecordingStatus {
            is_recording: false,
            session_id: None,
            elapsed_seconds: 0.0,
            click_count: 0,
            segment_count: 0,
            debug_mode: false,
            live_zoom: false,
            live_magnifier: false,
            record_system_audio: true,
            record_mic: true,
        }
    }
}

#[tauri::command]
pub fn start_recording(
    app_handle: AppHandle,
    debug_mode: bool,
    live_zoom: Option<bool>,
    live_magnifier: Option<bool>,
    record_system_audio: Option<bool>,
    record_mic: Option<bool>,
    monitor_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut session_guard = state.active_session.lock();
    if session_guard.is_some() {
        return Err("Recording is already in progress.".to_string());
    }
    let live_zoom = live_zoom.unwrap_or(false);
    let live_magnifier = live_magnifier.unwrap_or(false);
    let record_system_audio = record_system_audio.unwrap_or(true);
    let record_mic = record_mic.unwrap_or(true);

    let clock = QpcClock::new();
    let qpc_start = clock.now();
    let session_id = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    let output_root = state.output_dir.lock().clone();
    let session_output_dir = output_root.clone();
    std::fs::create_dir_all(&session_output_dir)
        .map_err(|e| format!("Failed to ensure designated directory: {:?}", e))?;

    let start_segment_index = encoder::get_next_segment_index(&session_output_dir, Some(&state.storage));

    // Resolve target display monitor (or fallback to primary)
    let (target_monitor, hmonitor) = monitors::resolve_monitor(monitor_id.as_deref());
    let offset_x = target_monitor.x;
    let offset_y = target_monitor.y;
    log::info!("Recording target display: {} (offset: {}, {})", target_monitor.name, offset_x, offset_y);

    // 1. Setup Frame Queue (Bounded to prevent memory leak, capacity of 60 frames = 1 second at 60fps)
    let (frame_sender, frame_receiver) = bounded(60);
    let is_capturing = Arc::new(AtomicBool::new(true));

    // 2. Start Capture Pipeline on the selected monitor
    let capture = CapturePipeline::new(Some(hmonitor), frame_sender, Arc::clone(&is_capturing))
        .map_err(|e| format!("Failed to initialize Windows.Graphics.Capture: {:?}", e))?;

    let width = capture.width;
    let height = capture.height;
    let fps = 60u32;

    // 3. Start Live Screen Magnifier if requested (` Key = Zoom In & Auto-Pan, Esc Key = Zoom Out)
    let magnifier_arc = if live_magnifier || live_zoom {
        match LiveMagnifierController::start(
            width,
            height,
            offset_x,
            offset_y,
            session_id.clone(),
            qpc_start,
            clock.frequency,
            Arc::clone(&state.storage),
            live_magnifier,
        ) {
            Ok(ctrl) => Some(Arc::new(ctrl)),
            Err(e) => {
                log::warn!("Live screen magnifier failed to start: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Sync latest configured hotkey bindings from SQLite settings
    let hotkeys = crate::commands::hotkeys::load_hotkeys_from_storage(&state.storage);
    crate::mouse_hook::keyboard::update_hotkey_bindings(
        hotkeys.zoom_in.vk_code,
        hotkeys.zoom_out.vk_code,
        hotkeys.stop_recording.vk_code,
    );

    let app_handle_stop = app_handle.clone();
    let arc_for_cb = magnifier_arc.clone();
    let hook_cb: Arc<dyn Fn(&str, i32, i32) + Send + Sync + 'static> = Arc::new(move |btn, x, y| {
        if btn == "zoom_in" || btn == "backtick" {
            if let Some(ref arc) = arc_for_cb {
                arc.zoom_in(x, y);
            }
        } else if btn == "zoom_out" || btn == "escape" {
            if let Some(ref arc) = arc_for_cb {
                arc.zoom_out();
            }
        } else if btn == "stop_recording" || btn == "f9" {
            let _ = app_handle_stop.emit("zentra://request-stop-recording", ());
        }
    });

    // 4. Start Mouse Hook (records clicks and listens for dynamic hotkeys)
    let mouse_hook = MouseHookController::start(
        session_id.clone(),
        qpc_start,
        clock.frequency,
        Arc::clone(&state.storage),
        offset_x,
        offset_y,
        Some(hook_cb),
    );

    // 5. Save Session Metadata to SQLite
    let session_record = SessionRecord {
        id: session_id.clone(),
        start_qpc: qpc_start,
        end_qpc: 0,
        qpc_frequency: clock.frequency,
        width,
        height,
        fps,
        output_dir: session_output_dir.to_string_lossy().to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        export_status: "recorded".to_string(),
    };
    state
        .storage
        .insert_session(&session_record)
        .map_err(|e| format!("Failed to save session record: {:?}", e))?;

    // Recent clicks provider for debug overlay
    let hook_clicks = {
        let recent = mouse_hook.get_recent_clicks();
        Arc::new(move || recent.clone())
    };

    // Rolling segment maximum duration (300 seconds = 5 minutes)
    let segment_max_duration_seconds = 300.0;

    // 6. Start Dual Audio Recording Pipeline (WASAPI Loopback + Microphone)
    let (audio_pipeline, audio_receiver) = crate::audio::AudioPipeline::start(
        clock,
        qpc_start,
        record_system_audio,
        record_mic,
    );

    // 7. Start Media Foundation Hardware Encoder
    let shared_transform_opt = magnifier_arc.as_ref().map(|m| m.shared_transform());
    let encoder = RollingEncoder::start(
        session_id.clone(),
        session_output_dir,
        start_segment_index,
        width,
        height,
        fps,
        clock,
        qpc_start,
        segment_max_duration_seconds,
        frame_receiver,
        audio_receiver,
        Arc::clone(&state.storage),
        debug_mode,
        live_zoom,
        capture.d3d11_device.clone(),
        hook_clicks,
        shared_transform_opt,
    )
    .map_err(|e| format!("Failed to start Media Foundation encoder: {:?}", e))?;

    *session_guard = Some(ActiveSession {
        session_id: session_id.clone(),
        start_qpc: qpc_start,
        qpc_frequency: clock.frequency,
        debug_mode,
        live_zoom,
        live_magnifier: magnifier_arc,
        record_system_audio,
        record_mic,
        capture,
        encoder,
        audio: audio_pipeline,
        mouse_hook,
    });

    // Hide main application window completely from screen
    if let Some(main) = app_handle.get_webview_window("main") {
        let _ = main.hide();
    }

    // Show floating widget with hardware screen-capture exclusion
    if let Some(widget) = app_handle.get_webview_window("widget") {
        apply_capture_exclusion(&widget);
        if !is_widget_on_screen(&widget) {
            position_widget_window(&widget);
        }
        let _ = widget.show();
        let _ = widget.set_always_on_top(true);
    }

    let _ = app_handle.emit("zentra://recording-started", ());
    crate::tray::update_tray_state(&app_handle, true);

    log::info!("Session {} started (debug_mode: {}, live_zoom: {}, live_magnifier: {}, record_system_audio: {}, record_mic: {})", session_id, debug_mode, live_zoom, live_magnifier, record_system_audio, record_mic);
    Ok(session_id)
}

#[tauri::command]
pub fn stop_recording(app_handle: AppHandle, state: State<'_, AppState>) -> Result<SessionSummary, String> {
    let mut session_guard = state.active_session.lock();
    let mut session = match session_guard.take() {
        Some(s) => s,
        None => {
            if let Some(widget) = app_handle.get_webview_window("widget") {
                let _ = widget.hide();
            }
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            crate::tray::update_tray_state(&app_handle, false);
            return Err("No active recording session to stop.".to_string());
        }
    };

    let clock = QpcClock::new();
    let qpc_end = clock.now();

    // 1. Stop and reset live screen magnifier immediately
    if let Some(ref mag) = session.live_magnifier {
        mag.stop_and_reset();
    }

    // 2. Stop capture
    session.capture.stop();

    // 3. Stop mouse hook
    session.mouse_hook.stop();

    // 4. Stop audio capture pipeline
    session.audio.stop();

    // 5. Stop encoder and flush remaining frames / finalize segment
    session.encoder.stop();

    // 5. Update session end timestamp in SQLite
    state
        .storage
        .finish_session(&session.session_id, qpc_end)
        .map_err(|e| format!("Failed to update session end: {:?}", e))?;

    // 6. Hide widget window
    if let Some(widget) = app_handle.get_webview_window("widget") {
        if let Ok(pos) = widget.outer_position() {
            let _ = state.storage.save_setting("widget_pos", &format!("{},{}", pos.x, pos.y));
        }
        let _ = widget.hide();
    }

    // 7. Restore and focus desktop application window
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    let _ = app_handle.emit("zentra://recording-stopped", ());
    crate::tray::update_tray_state(&app_handle, false);

    let summary_list = state
        .storage
        .list_sessions()
        .map_err(|e| format!("Failed to list sessions: {:?}", e))?;

    let summary = summary_list
        .into_iter()
        .find(|s| s.id == session.session_id)
        .ok_or_else(|| "Session summary not found".to_string())?;

    log::info!("Session {} stopped successfully.", session.session_id);
    Ok(summary)
}