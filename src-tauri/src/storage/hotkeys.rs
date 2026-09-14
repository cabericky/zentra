use serde::{Deserialize, Serialize};

use super::Storage;

pub const HOTKEYS_SETTING_KEY: &str = "hotkeys";

/// A single keyboard hotkey binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub vk_code: u32,
    pub key: String,
    pub label: String,
}

/// Hotkey configuration for all interactive shortcut actions in Zentra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub zoom_in: HotkeyBinding,
    pub zoom_out: HotkeyBinding,
    pub stop_recording: HotkeyBinding,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            zoom_in: HotkeyBinding {
                vk_code: 0xC0, // VK_OEM_3 (`~)
                key: "Backquote".to_string(),
                label: "` (Backtick)".to_string(),
            },
            zoom_out: HotkeyBinding {
                vk_code: 0x1B, // VK_ESCAPE
                key: "Escape".to_string(),
                label: "Esc".to_string(),
            },
            stop_recording: HotkeyBinding {
                vk_code: 0x78, // VK_F9
                key: "F9".to_string(),
                label: "F9".to_string(),
            },
        }
    }
}

/// Validates that hotkey bindings are non-empty and have no duplicate keys.
pub fn validate_hotkeys(settings: &HotkeySettings) -> Result<(), String> {
    if settings.zoom_in.vk_code == 0
        || settings.zoom_out.vk_code == 0
        || settings.stop_recording.vk_code == 0
    {
        return Err("Hotkeys cannot be empty or unbound.".to_string());
    }

    if settings.zoom_in.vk_code == settings.zoom_out.vk_code {
        return Err("Zoom In and Zoom Out cannot use the same key.".to_string());
    }
    if settings.zoom_in.vk_code == settings.stop_recording.vk_code {
        return Err("Zoom In and Stop Recording cannot use the same key.".to_string());
    }
    if settings.zoom_out.vk_code == settings.stop_recording.vk_code {
        return Err("Zoom Out and Stop Recording cannot use the same key.".to_string());
    }

    Ok(())
}

/// Loads configured hotkeys from SQLite storage, falling back to defaults if not set.
pub fn load_hotkeys_from_storage(storage: &Storage) -> HotkeySettings {
    if let Ok(Some(json_str)) = storage.get_setting(HOTKEYS_SETTING_KEY) {
        if let Ok(parsed) = serde_json::from_str::<HotkeySettings>(&json_str) {
            return parsed;
        }
    }
    HotkeySettings::default()
}

/// Persists hotkeys into SQLite storage after validation.
pub fn persist_hotkeys_to_storage(storage: &Storage, settings: &HotkeySettings) -> Result<(), String> {
    validate_hotkeys(settings)?;
    let json = serde_json::to_string(settings)
        .map_err(|e| format!("Failed to serialize hotkeys: {}", e))?;
    storage
        .save_setting(HOTKEYS_SETTING_KEY, &json)
        .map_err(|e| format!("Failed to save hotkeys to SQLite settings: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hotkeys_valid() {
        let defaults = HotkeySettings::default();
        assert!(validate_hotkeys(&defaults).is_ok());
        assert_eq!(defaults.zoom_in.vk_code, 0xC0);
        assert_eq!(defaults.zoom_out.vk_code, 0x1B);
        assert_eq!(defaults.stop_recording.vk_code, 0x78);
    }

    #[test]
    fn test_duplicate_hotkey_validation_fails() {
        let mut settings = HotkeySettings::default();
        settings.zoom_in.vk_code = settings.zoom_out.vk_code;
        assert!(validate_hotkeys(&settings).is_err());
    }

    #[test]
    fn test_zero_hotkey_validation_fails() {
        let mut settings = HotkeySettings::default();
        settings.zoom_in.vk_code = 0;
        assert!(validate_hotkeys(&settings).is_err());
    }

    #[test]
    fn test_storage_persistence() {
        let temp_db = std::env::temp_dir().join(format!(
            "test_zentra_hotkeys_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let storage = Storage::new(&temp_db).unwrap();
        let initial = load_hotkeys_from_storage(&storage);
        assert_eq!(initial, HotkeySettings::default());

        let mut custom = HotkeySettings::default();
        custom.zoom_in.vk_code = 0x77; // F8
        custom.zoom_in.key = "F8".to_string();
        custom.zoom_in.label = "F8".to_string();

        assert!(persist_hotkeys_to_storage(&storage, &custom).is_ok());
        let reloaded = load_hotkeys_from_storage(&storage);
        assert_eq!(reloaded.zoom_in.vk_code, 0x77);
        assert_eq!(reloaded.zoom_in.label, "F8");

        let _ = std::fs::remove_file(&temp_db);
    }
}