use std::path::PathBuf;
use tauri::State;
use crate::folders::service::{self, record_known_folder};
use crate::state::{AppState, DesignatedFolderInfo, FolderItem};

pub use crate::folders::service::{remove_known_folder, update_known_folder};

#[tauri::command]
pub fn get_designated_folder_info(state: State<'_, AppState>) -> DesignatedFolderInfo {
    let current_p = state.output_dir.lock().clone();
    let name = current_p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Zentra".to_string());
    let exists = current_p.exists() && current_p.is_dir();
    DesignatedFolderInfo {
        current_path: current_p.to_string_lossy().to_string(),
        current_name: name,
        base_path: state.base_dir.to_string_lossy().to_string(),
        exists,
    }
}

#[tauri::command]
pub fn set_designated_folder(path: String, state: State<'_, AppState>) -> Result<DesignatedFolderInfo, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Folder path cannot be empty.".to_string());
    }
    let p = PathBuf::from(trimmed);
    if !p.exists() {
        return Err(format!("Folder does not exist: {}", p.display()));
    }
    if !p.is_dir() {
        return Err(format!("Specified path is not a folder: {}", p.display()));
    }

    *state.output_dir.lock() = p.clone();
    let _ = state.storage.save_setting("designated_folder", &p.to_string_lossy());
    record_known_folder(&state.storage, &p);

    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Folder".to_string());

    Ok(DesignatedFolderInfo {
        current_path: p.to_string_lossy().to_string(),
        current_name: name,
        base_path: state.base_dir.to_string_lossy().to_string(),
        exists: true,
    })
}

#[tauri::command]
pub fn create_designated_folder(name_or_path: String, state: State<'_, AppState>) -> Result<DesignatedFolderInfo, String> {
    let p = service::create_folder(&state.base_dir, &name_or_path)?;

    *state.output_dir.lock() = p.clone();
    let _ = state.storage.save_setting("designated_folder", &p.to_string_lossy());
    record_known_folder(&state.storage, &p);

    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| name_or_path.trim().to_string());

    Ok(DesignatedFolderInfo {
        current_path: p.to_string_lossy().to_string(),
        current_name: name,
        base_path: state.base_dir.to_string_lossy().to_string(),
        exists: true,
    })
}

#[tauri::command]
pub fn list_available_folders(state: State<'_, AppState>) -> Result<Vec<FolderItem>, String> {
    let current_dir = state.output_dir.lock().clone();
    service::list_available_folders(&state.storage, &state.base_dir, &current_dir)
}

#[tauri::command]
pub fn browse_for_folder(state: State<'_, AppState>) -> Result<Option<DesignatedFolderInfo>, String> {
    let opt_p = service::browse_for_folder()?;
    if let Some(p) = opt_p {
        let path_str = p.to_string_lossy().to_string();
        *state.output_dir.lock() = p.clone();
        let _ = state.storage.save_setting("designated_folder", &path_str);
        record_known_folder(&state.storage, &p);

        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        Ok(Some(DesignatedFolderInfo {
            current_path: path_str,
            current_name: name,
            base_path: state.base_dir.to_string_lossy().to_string(),
            exists: true,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn get_output_folder(state: State<'_, AppState>) -> String {
    state.output_dir.lock().to_string_lossy().to_string()
}

#[tauri::command]
pub fn set_output_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let p = PathBuf::from(path.trim());
    if !p.exists() {
        std::fs::create_dir_all(&p).map_err(|e| format!("Failed to create directory: {:?}", e))?;
    }
    *state.output_dir.lock() = p.clone();
    let _ = state.storage.save_setting("designated_folder", &p.to_string_lossy());
    record_known_folder(&state.storage, &p);
    Ok(())
}

#[tauri::command]
pub fn open_output_folder(state: State<'_, AppState>) -> Result<(), String> {
    let folder = state.output_dir.lock().clone();
    service::open_folder_in_explorer(&folder)
}

#[tauri::command]
pub fn rename_designated_folder(
    old_path: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<DesignatedFolderInfo, String> {
    let old_p = PathBuf::from(old_path.trim());
    let new_p = service::rename_folder(&state.storage, &state.base_dir, &old_path, &new_name)?;

    let mut cur_guard = state.output_dir.lock();
    if *cur_guard == old_p {
        *cur_guard = new_p.clone();
        let _ = state.storage.save_setting("designated_folder", &new_p.to_string_lossy());
    }

    let actual_name = new_p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| new_name.trim().to_string());

    let new_str = new_p.to_string_lossy().to_string();
    log::info!("Folder renamed from '{}' to '{}'", old_path, new_str);
    Ok(DesignatedFolderInfo {
        current_path: new_str,
        current_name: actual_name,
        base_path: state.base_dir.to_string_lossy().to_string(),
        exists: true,
    })
}

#[tauri::command]
pub fn delete_designated_folder(
    path: String,
    state: State<'_, AppState>,
) -> Result<DesignatedFolderInfo, String> {
    let p = PathBuf::from(path.trim());
    service::delete_folder(&state.storage, &state.base_dir, &path)?;

    let mut cur_guard = state.output_dir.lock();
    if *cur_guard == p {
        *cur_guard = state.base_dir.clone();
        let _ = state.storage.save_setting("designated_folder", &state.base_dir.to_string_lossy());
    }

    let cur = cur_guard.clone();
    let name = cur
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Zentra".to_string());

    log::info!("Folder '{}' deleted successfully.", path);
    Ok(DesignatedFolderInfo {
        current_path: cur.to_string_lossy().to_string(),
        current_name: name,
        base_path: state.base_dir.to_string_lossy().to_string(),
        exists: cur.exists(),
    })
}