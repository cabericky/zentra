pub mod audio;
pub mod capture;
pub mod clock;
pub mod commands;
pub mod debug_overlay;
pub mod encoder;
pub mod export;
pub mod folders;
pub mod live_magnifier;
pub mod mouse_hook;
pub mod monitors;
pub mod platform;
pub mod state;
pub mod storage;
pub mod tray;
pub mod window_utils;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::Manager;

use commands::*;
use state::AppState;
use storage::Storage;
use window_utils::{apply_capture_exclusion, is_widget_on_screen, position_widget_window, timeBeginPeriod};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        crate::live_magnifier::LiveMagnifierController::reset_display();
        default_panic_hook(info);
    }));

    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware();
        let _ = timeBeginPeriod(1);
    }

    let videos_dir = dirs::video_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\"))
        .join("Zentra");
    let _ = std::fs::create_dir_all(&videos_dir);

    let db_path = videos_dir.join("zentra.db");
    let storage = Arc::new(Storage::new(db_path).expect("failed to initialize SQLite storage"));
    let storage_for_setup = Arc::clone(&storage);

    let mut initial_output_dir = videos_dir.clone();
    if let Ok(Some(saved_dir)) = storage.get_setting("designated_folder") {
        let p = PathBuf::from(saved_dir);
        if p.exists() && p.is_dir() {
            initial_output_dir = p;
        }
    }

    let initial_hotkeys = commands::hotkeys::load_hotkeys_from_storage(&storage);
    mouse_hook::keyboard::update_hotkey_bindings(
        initial_hotkeys.zoom_in.vk_code,
        initial_hotkeys.zoom_out.vk_code,
        initial_hotkeys.stop_recording.vk_code,
    );

    let app_state = AppState {
        storage,
        active_session: Arc::new(Mutex::new(None)),
        base_dir: videos_dir,
        output_dir: Arc::new(Mutex::new(initial_output_dir)),
        export_cancel_flag: Arc::new(AtomicBool::new(false)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            if let Some(widget) = app.get_webview_window("widget") {
                apply_capture_exclusion(&widget);
                let mut restored = false;
                if let Ok(Some(pos_str)) = storage_for_setup.get_setting("widget_pos") {
                    let parts: Vec<&str> = pos_str.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                            let _ = widget.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                            if is_widget_on_screen(&widget) {
                                restored = true;
                            }
                        }
                    }
                }
                if !restored {
                    position_widget_window(&widget);
                }
            }
            if let Err(e) = tray::setup_tray(app) {
                log::error!("Failed to initialize Windows system tray: {:?}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                    log::info!("Main window hidden to system tray on close request.");
                }
            }
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            minimize_window,
            restore_window,
            hide_main_window,
            show_main_window,
            show_widget_window,
            hide_widget_window,
            start_widget_drag,
            get_widget_position,
            set_widget_position,
            reset_screen_magnifier,
            get_available_monitors,
            get_recording_status,
            list_sessions,
            export_session,
            cancel_export,
            open_output_folder,
            open_folder,
            open_file,
            get_output_folder,
            set_output_folder,
            get_designated_folder_info,
            set_designated_folder,
            create_designated_folder,
            list_available_folders,
            browse_for_folder,
            delete_session,
            rename_session,
            rename_designated_folder,
            delete_designated_folder,
            get_hotkeys,
            save_hotkeys,
            reset_hotkeys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zentra application");
}