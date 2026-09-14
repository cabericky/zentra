use std::path::Path;
use rusqlite::Result;
use super::models::SessionSummary;
use super::Storage;

/// Reconciles database records against the physical filesystem on disk:
/// 1. If a folder was deleted on disk, removes its session records from SQLite.
/// 2. If a folder was renamed externally under base_dir, heals paths to match the new folder.
/// 3. If a session's video files were deleted on disk, removes the session record.
/// 4. If a single segment file was renamed on disk, updates the record's file_path.
pub fn reconcile_and_list_sessions(storage: &Storage, base_dir: &Path) -> Result<Vec<SessionSummary>> {
    let raw_sessions = storage.list_sessions()?;
    let mut sessions_to_delete = Vec::new();

    for s in &raw_sessions {
        let out_dir = Path::new(&s.output_dir);
        if !out_dir.exists() {
            // Check if directory was renamed under base_dir
            let mut found_renamed = false;
            if let Ok(entries) = std::fs::read_dir(base_dir) {
                for entry in entries.flatten() {
                    let candidate = entry.path();
                    if candidate.is_dir() && candidate != out_dir {
                        if let Ok(segs) = storage.get_segments(&s.id) {
                            if let Some(first_seg) = segs.first() {
                                if let Some(file_name) = Path::new(&first_seg.file_path).file_name() {
                                    if candidate.join(file_name).exists() {
                                        let _ = storage.rename_folder_in_records(&s.output_dir, &candidate.to_string_lossy());
                                        found_renamed = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !found_renamed {
                sessions_to_delete.push(s.id.clone());
            }
        } else {
            // Directory exists: verify segment files or zoom export files exist
            if let Ok(segs) = storage.get_segments(&s.id) {
                let mut any_file_found = false;
                for seg in &segs {
                    if Path::new(&seg.file_path).exists() {
                        any_file_found = true;
                        break;
                    }
                }

                if !any_file_found {
                    if let Some(ref exp_name) = s.export_file_name {
                        if out_dir.join(exp_name).exists() {
                            any_file_found = true;
                        }
                    }
                }

                if !any_file_found {
                    // Check if a single segment was renamed on disk
                    let mut auto_matched = false;
                    if segs.len() == 1 {
                        if let Ok(files) = std::fs::read_dir(out_dir) {
                            for f in files.flatten() {
                                let fp = f.path();
                                if fp.is_file() && fp.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("mp4")) {
                                    let name = fp.file_name().unwrap_or_default().to_string_lossy();
                                    if !name.ends_with("_zoom.mp4") && !name.starts_with("zentra_export_") {
                                        let _ = storage.update_segment_file_path(&s.id, segs[0].segment_index, &fp.to_string_lossy());
                                        auto_matched = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !auto_matched {
                        sessions_to_delete.push(s.id.clone());
                    }
                }
            }
        }
    }

    for sid in sessions_to_delete {
        let _ = storage.delete_session(&sid);
    }

    storage.list_sessions()
}