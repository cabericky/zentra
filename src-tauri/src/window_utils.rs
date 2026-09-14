use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
};

pub use crate::platform::timeBeginPeriod;
pub use crate::platform::is_windows_build_at_least;

pub fn apply_capture_exclusion(window: &tauri::WebviewWindow) {
    let _ = window.set_shadow(false);
    if let Ok(raw_hwnd) = window.hwnd() {
        let hwnd = HWND(raw_hwnd.0 as _);
        unsafe {
            let res = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            log::info!("Applied WDA_EXCLUDEFROMCAPTURE to window '{}': {:?}", window.label(), res);
        }
    }
}

pub fn position_widget_window(widget: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = widget.primary_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let widget_width_px = (340.0 * scale) as i32;
        let x = ((size.width as i32) - widget_width_px) / 2;
        let y = (28.0 * scale) as i32;
        let _ = widget.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
    }
}

pub fn is_widget_on_screen(widget: &tauri::WebviewWindow) -> bool {
    if let Ok(pos) = widget.outer_position() {
        if let Ok(monitors) = widget.available_monitors() {
            for m in monitors {
                let m_pos = m.position();
                let m_size = m.size();
                if pos.x >= m_pos.x - 100
                    && pos.x < m_pos.x + (m_size.width as i32)
                    && pos.y >= m_pos.y
                    && pos.y < m_pos.y + (m_size.height as i32)
                {
                    return true;
                }
            }
        }
    }
    false
}