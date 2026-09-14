pub mod types;
pub mod win32;

pub use types::MonitorInfo;
pub use win32::query_connected_monitors as enumerate_monitors;

use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, HMONITOR, MONITOR_DEFAULTTOPRIMARY};

/// Resolves a requested monitor by ID/handle or falls back to the primary display.
pub fn resolve_monitor(target_id: Option<&str>) -> (MonitorInfo, HMONITOR) {
    let monitors = enumerate_monitors();

    if let Some(target) = target_id {
        if let Some(found) = monitors
            .iter()
            .find(|m| m.id == target || m.hmonitor.to_string() == target)
        {
            return (found.clone(), HMONITOR(found.hmonitor as *mut _));
        }
    }

    if let Some(primary) = monitors
        .iter()
        .find(|m| m.is_primary)
        .or_else(|| monitors.first())
    {
        return (primary.clone(), HMONITOR(primary.hmonitor as *mut _));
    }

    let hmon = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
    let fallback = MonitorInfo {
        id: r"\\.\DISPLAY1".to_string(),
        name: "Primary Display — 1920×1080".to_string(),
        hmonitor: hmon.0 as isize,
        is_primary: true,
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };

    (fallback, hmon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_monitors() {
        let monitors = enumerate_monitors();
        assert!(!monitors.is_empty(), "Should enumerate at least one monitor");
        assert!(
            monitors.iter().any(|m| m.is_primary),
            "At least one monitor should be marked as primary"
        );
        for m in &monitors {
            assert!(m.width > 0, "Monitor width must be greater than 0");
            assert!(m.height > 0, "Monitor height must be greater than 0");
        }
    }

    #[test]
    fn test_resolve_monitor_fallback() {
        let (resolved, hmon) = resolve_monitor(None);
        assert!(!resolved.id.is_empty());
        assert!(!hmon.0.is_null());
    }
}