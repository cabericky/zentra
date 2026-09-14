use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::state::AppState;
use super::actions::{restore_main_window, show_or_reposition_widget};
use super::types::*;

pub fn handle_tray_icon_event(tray: &TrayIcon<tauri::Wry>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            restore_main_window(tray.app_handle());
        }
        _ => {}
    }
}

pub fn handle_tray_menu_event(app: &AppHandle, menu_id: &str) {
    match menu_id {
        MENU_TOGGLE => {
            let state = app.state::<AppState>();
            let is_recording = state.active_session.lock().is_some();
            if is_recording {
                match crate::commands::recording::stop_recording(app.clone(), state) {
                    Ok(_) => {
                        log::info!("Recording stopped via system tray.");
                    }
                    Err(e) => {
                        log::error!("Failed to stop recording via system tray: {}", e);
                    }
                }
            } else {
                match crate::commands::recording::start_recording(
                    app.clone(),
                    false,
                    Some(false),
                    Some(true),
                    Some(true),
                    Some(true),
                    None,
                    state,
                ) {
                    Ok(sid) => {
                        log::info!("Recording session {} started via system tray.", sid);
                    }
                    Err(e) => {
                        log::error!("Failed to start recording via system tray: {}", e);
                    }
                }
            }
        }
        MENU_SHOW_MAIN => {
            restore_main_window(app);
        }
        MENU_SHOW_WIDGET => {
            show_or_reposition_widget(app);
        }
        MENU_SETTINGS => {
            restore_main_window(app);
            let _ = app.emit("zentra://focus-settings", ());
        }
        MENU_EXIT => {
            let state = app.state::<AppState>();
            if state.active_session.lock().is_some() {
                let _ = crate::commands::recording::stop_recording(app.clone(), state);
            }
            app.exit(0);
        }
        _ => {}
    }
}