use tauri::menu::MenuItem;

pub const TRAY_ID: &str = "zentra_tray";
pub const MENU_STATUS: &str = "tray_status";
pub const MENU_TOGGLE: &str = "tray_toggle_recording";
pub const MENU_SHOW_MAIN: &str = "tray_show_main";
pub const MENU_SHOW_WIDGET: &str = "tray_show_widget";
pub const MENU_SETTINGS: &str = "tray_settings";
pub const MENU_EXIT: &str = "tray_exit";

pub struct TrayState {
    pub status_item: MenuItem<tauri::Wry>,
    pub toggle_item: MenuItem<tauri::Wry>,
}