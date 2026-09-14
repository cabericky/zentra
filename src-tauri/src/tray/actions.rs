use tauri::{AppHandle, Manager};
use crate::window_utils::{apply_capture_exclusion, position_widget_window};

pub fn restore_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn show_or_reposition_widget(app: &AppHandle) {
    if let Some(widget) = app.get_webview_window("widget") {
        apply_capture_exclusion(&widget);
        position_widget_window(&widget);
        let _ = widget.show();
        let _ = widget.set_always_on_top(true);
        let _ = widget.set_focus();
    }
}