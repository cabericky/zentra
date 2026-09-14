use crate::monitors::{self, MonitorInfo};

#[tauri::command]
pub fn get_available_monitors() -> Vec<MonitorInfo> {
    monitors::enumerate_monitors()
}