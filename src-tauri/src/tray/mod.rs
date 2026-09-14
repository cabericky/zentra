pub mod actions;
pub mod events;
pub mod menu;
pub mod types;

pub use actions::{restore_main_window, show_or_reposition_widget};
pub use types::{TrayState, TRAY_ID};

use tauri::{tray::TrayIconBuilder, AppHandle, Manager};

pub fn update_tray_state(app: &AppHandle, is_recording: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = if is_recording {
            "Zentra - Recording Screen..."
        } else {
            "Zentra - Offline Screen Recorder (Idle)"
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    if let Some(tray_state) = app.try_state::<TrayState>() {
        if is_recording {
            let _ = tray_state.status_item.set_text("Status: Recording...");
            let _ = tray_state.toggle_item.set_text("Stop Recording (F9)");
        } else {
            let _ = tray_state.status_item.set_text("Status: Idle");
            let _ = tray_state.toggle_item.set_text("Start Recording");
        }
    }
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let (menu, status_item, toggle_item) = menu::build_tray_menu(app_handle)?;
    let icon = menu::load_tray_icon(app_handle);

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Zentra - Offline Screen Recorder (Idle)")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            events::handle_tray_icon_event(tray, event);
        })
        .on_menu_event(|app, event| {
            events::handle_tray_menu_event(app, event.id.as_ref());
        })
        .build(app_handle)?;

    app.manage(TrayState {
        status_item,
        toggle_item,
    });

    Ok(())
}