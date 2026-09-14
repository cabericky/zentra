use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::export::SessionExporter;
use crate::state::AppState;
use crate::storage::SessionSummary;

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>, String> {
    state
        .storage
        .reconcile_and_list_sessions(&state.base_dir)
        .map_err(|e| format!("Failed to fetch sessions: {:?}", e))
}

#[tauri::command]
pub fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let session = state
        .storage
        .get_session(&session_id)
        .map_err(|e| format!("Failed to get session: {:?}", e))?;

    let segments = state
        .storage
        .get_segments(&session_id)
        .unwrap_or_default();

    let out_dir = PathBuf::from(&session.output_dir);

    // Delete segment video files and corresponding zoom exports from disk
    for seg in &segments {
        let p = PathBuf::from(&seg.file_path);
        if p.exists() && p.is_file() {
            let _ = std::fs::remove_file(&p);
        }

        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            let zoom_path = out_dir.join(format!("{}_zoom.mp4", stem));
            if zoom_path.exists() && zoom_path.is_file() {
                let _ = std::fs::remove_file(&zoom_path);
            }
        }
    }

    // Also check for legacy export name
    let legacy_zoom = out_dir.join(format!("zentra_export_{}.mp4", session_id));
    if legacy_zoom.exists() && legacy_zoom.is_file() {
        let _ = std::fs::remove_file(&legacy_zoom);
    }

    // Delete session records from SQLite database
    state
        .storage
        .delete_session(&session_id)
        .map_err(|e| format!("Failed to delete session records: {:?}", e))?;

    // If session was in a legacy auto-generated subfolder and now empty, clean it up
    let fname = out_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let is_session_dir = fname.len() == 15
        && fname.chars().take(8).all(|c| c.is_ascii_digit())
        && fname.chars().nth(8) == Some('_')
        && fname.chars().skip(9).take(6).all(|c| c.is_ascii_digit());
    if is_session_dir && out_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&out_dir) {
            if entries.count() == 0 {
                let _ = std::fs::remove_dir(&out_dir);
            }
        }
    }

    log::info!("Session {} and its files deleted successfully.", session_id);
    Ok(())
}

#[tauri::command]
pub fn rename_session(session_id: String, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut clean_name = new_name.trim().to_string();
    if clean_name.is_empty() {
        return Err("New session name cannot be empty.".to_string());
    }
    if clean_name.contains('/')
        || clean_name.contains('\\')
        || clean_name.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    {
        return Err("Session name contains invalid characters.".to_string());
    }
    if clean_name.to_lowercase().ends_with(".mp4") {
        clean_name = clean_name[..clean_name.len() - 4].to_string();
    }
    let clean_stem = clean_name.trim();
    if clean_stem.is_empty() {
        return Err("Invalid session name.".to_string());
    }

    let session = state
        .storage
        .get_session(&session_id)
        .map_err(|e| format!("Session not found: {:?}", e))?;

    let segments = state
        .storage
        .get_segments(&session_id)
        .map_err(|e| format!("Failed to get session segments: {:?}", e))?;

    if segments.is_empty() {
        return Err("Session has no segment records to rename.".to_string());
    }

    let out_dir = PathBuf::from(&session.output_dir);

    for (idx, seg) in segments.iter().enumerate() {
        let old_p = PathBuf::from(&seg.file_path);
        let parent_dir = old_p.parent().unwrap_or(&out_dir);

        let target_filename = if segments.len() == 1 {
            format!("{}.mp4", clean_stem)
        } else {
            format!("{}_part{:02}.mp4", clean_stem, idx + 1)
        };
        let new_p = parent_dir.join(&target_filename);

        if new_p.exists() && new_p != old_p {
            return Err(format!("A file named '{}' already exists in this folder.", target_filename));
        }

        // Rename file on disk if it exists
        if old_p.exists() {
            std::fs::rename(&old_p, &new_p).map_err(|e| format!("Failed to rename video file: {:?}", e))?;
        }

        // Rename zoom export file if exists
        let old_stem = old_p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let old_zoom = parent_dir.join(format!("{}_zoom.mp4", old_stem));
        let new_zoom_filename = if segments.len() == 1 {
            format!("{}_zoom.mp4", clean_stem)
        } else {
            format!("{}_part{:02}_zoom.mp4", clean_stem, idx + 1)
        };
        let new_zoom = parent_dir.join(&new_zoom_filename);
        if old_zoom.exists() {
            let _ = std::fs::rename(&old_zoom, &new_zoom);
        }

        // Check legacy export file
        let legacy_zoom = parent_dir.join(format!("zentra_export_{}.mp4", session_id));
        if legacy_zoom.exists() && !new_zoom.exists() {
            let _ = std::fs::rename(&legacy_zoom, &new_zoom);
        }

        // Update SQLite record
        state
            .storage
            .update_segment_file_path(&session_id, seg.segment_index, &new_p.to_string_lossy())
            .map_err(|e| format!("Failed to update database segment record: {:?}", e))?;
    }

    log::info!("Session {} renamed to '{}' successfully.", session_id, clean_stem);
    Ok(())
}

#[tauri::command]
pub fn cancel_export(state: State<'_, AppState>) {
    state.export_cancel_flag.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub async fn export_session(
    session_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let storage = Arc::clone(&state.storage);
    let cancel_flag = Arc::clone(&state.export_cancel_flag);
    cancel_flag.store(false, Ordering::SeqCst);

    let sid = session_id.clone();
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn_blocking(move || {
        SessionExporter::export_session(app_handle_clone, sid, storage, cancel_flag)
    })
    .await
    .map_err(|e| format!("Task join error: {:?}", e))?
    .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        let _ = std::fs::create_dir_all(&p);
    }
    std::process::Command::new("explorer.exe")
        .arg(p)
        .spawn()
        .map_err(|e| format!("Failed to open directory: {:?}", e))?;
    Ok(())
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    std::process::Command::new("explorer.exe")
        .arg(format!("/select,{}", p.display()))
        .spawn()
        .map_err(|e| format!("Failed to highlight file in explorer: {:?}", e))?;
    Ok(())
}