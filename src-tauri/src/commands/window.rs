use tauri::{AppHandle, Manager, State};
use crate::state::AppState;
use crate::window_utils::{apply_capture_exclusion, is_widget_on_screen, position_widget_window};

#[tauri::command]
pub fn minimize_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
pub fn restore_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn hide_main_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn show_widget_window(app_handle: AppHandle) {
    if let Some(widget) = app_handle.get_webview_window("widget") {
        apply_capture_exclusion(&widget);
        if !is_widget_on_screen(&widget) {
            position_widget_window(&widget);
        }
        let _ = widget.show();
        let _ = widget.set_always_on_top(true);
    }
}

#[tauri::command]
pub fn hide_widget_window(app_handle: AppHandle, state: State<'_, AppState>) {
    if let Some(widget) = app_handle.get_webview_window("widget") {
        if let Ok(pos) = widget.outer_position() {
            let _ = state.storage.save_setting("widget_pos", &format!("{},{}", pos.x, pos.y));
        }
        let _ = widget.hide();
    }
}

#[tauri::command]
pub fn start_widget_drag(app_handle: AppHandle) {
    if let Some(widget) = app_handle.get_webview_window("widget") {
        let _ = widget.start_dragging();
    }
}

#[tauri::command]
pub fn get_widget_position(app_handle: AppHandle) -> Option<(i32, i32)> {
    if let Some(widget) = app_handle.get_webview_window("widget") {
        if let Ok(pos) = widget.outer_position() {
            return Some((pos.x, pos.y));
        }
    }
    None
}

#[tauri::command]
pub fn set_widget_position(app_handle: AppHandle, state: State<'_, AppState>, x: i32, y: i32) {
    if let Some(widget) = app_handle.get_webview_window("widget") {
        let _ = widget.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        let _ = state.storage.save_setting("widget_pos", &format!("{},{}", x, y));
    }
}