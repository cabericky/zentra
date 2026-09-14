use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW,
    DISPLAY_DEVICEW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};

use super::types::MonitorInfo;

/// Enumerates all currently connected active display monitors via Win32 EnumDisplayMonitors.
pub fn query_connected_monitors() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    let lparam = LPARAM(&mut monitors as *mut _ as isize);

    unsafe {
        let _ = EnumDisplayMonitors(
            HDC(std::ptr::null_mut()),
            None,
            Some(enum_monitor_callback),
            lparam,
        );
    }

    // Sort monitors so primary display comes first, then secondary displays by X coordinate
    monitors.sort_by(|a, b| {
        b.is_primary.cmp(&a.is_primary).then_with(|| a.x.cmp(&b.x))
    });

    monitors
}

/// Win32 callback function invoked for each monitor enumerated by EnumDisplayMonitors.
unsafe extern "system" fn enum_monitor_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    let ok = GetMonitorInfoW(
        hmonitor,
        &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
    );

    if ok.as_bool() {
        let rc = info.monitorInfo.rcMonitor;
        let width = (rc.right - rc.left).abs() as u32;
        let height = (rc.bottom - rc.top).abs() as u32;
        let is_primary = (info.monitorInfo.dwFlags & 1) != 0;

        let len = info
            .szDevice
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(info.szDevice.len());
        let device_id = String::from_utf16_lossy(&info.szDevice[..len]);

        let friendly_name = get_display_friendly_name(&info.szDevice).unwrap_or_else(|| {
            let index = monitors.len() + 1;
            format!("Display {}", index)
        });

        let display_label = if is_primary {
            format!("{} (Primary) — {}×{}", friendly_name, width, height)
        } else {
            format!("{} — {}×{}", friendly_name, width, height)
        };

        monitors.push(MonitorInfo {
            id: device_id,
            name: display_label,
            hmonitor: hmonitor.0 as isize,
            is_primary,
            x: rc.left,
            y: rc.top,
            width,
            height,
        });
    }

    BOOL(1)
}

/// Queries user-friendly display adapter / monitor name via Win32 EnumDisplayDevicesW.
unsafe fn get_display_friendly_name(device_name_raw: &[u16; 32]) -> Option<String> {
    let mut dd = DISPLAY_DEVICEW::default();
    dd.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

    let ok = EnumDisplayDevicesW(
        windows::core::PCWSTR(device_name_raw.as_ptr()),
        0,
        &mut dd,
        0,
    );

    if ok.as_bool() {
        let len = dd
            .DeviceString
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(dd.DeviceString.len());
        let name = String::from_utf16_lossy(&dd.DeviceString[..len])
            .trim()
            .to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }

    None
}