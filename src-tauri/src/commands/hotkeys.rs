use tauri::{AppHandle, Emitter, State};

use crate::mouse_hook::keyboard::update_hotkey_bindings;
use crate::state::AppState;
pub use crate::storage::hotkeys::*;

/// Retrieves the current hotkey configuration from storage.
#[tauri::command]
pub fn get_hotkeys(state: State<'_, AppState>) -> Result<HotkeySettings, String> {
    Ok(load_hotkeys_from_storage(&state.storage))
}

/// Saves and updates the hotkey configuration, dynamically applying it to the keyboard hook.
#[tauri::command]
pub fn save_hotkeys(
    hotkeys: HotkeySettings,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<HotkeySettings, String> {
    persist_hotkeys_to_storage(&state.storage, &hotkeys)?;
    update_hotkey_bindings(
        hotkeys.zoom_in.vk_code,
        hotkeys.zoom_out.vk_code,
        hotkeys.stop_recording.vk_code,
    );
    let _ = app.emit("zentra://hotkeys-updated", &hotkeys);
    Ok(hotkeys)
}

/// Resets hotkey configuration to factory defaults.
#[tauri::command]
pub fn reset_hotkeys(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<HotkeySettings, String> {
    let defaults = HotkeySettings::default();
    persist_hotkeys_to_storage(&state.storage, &defaults)?;
    update_hotkey_bindings(
        defaults.zoom_in.vk_code,
        defaults.zoom_out.vk_code,
        defaults.stop_recording.vk_code,
    );
    let _ = app.emit("zentra://hotkeys-updated", &defaults);
    Ok(defaults)
}