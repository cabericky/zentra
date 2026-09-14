use std::path::Path;
use crate::storage::Storage;

/// Scans the output directory and storage records to determine the next contiguous segment index.
/// Numbers continue sequentially (e.g. segment_0001, segment_0002, ...) in the designated folder.
pub fn get_next_segment_index(dir: &Path, storage: Option<&Storage>) -> u32 {
    let mut max_idx = 0u32;
    let mut found_any = false;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(rest) = stem.strip_prefix("segment_") {
                    let num_part = rest.split('_').next().unwrap_or(rest);
                    if let Ok(idx) = num_part.parse::<u32>() {
                        if !found_any || idx > max_idx {
                            max_idx = idx;
                        }
                        found_any = true;
                    }
                }
            }
        }
    }

    if let Some(storage) = storage {
        if let Ok(sessions) = storage.list_sessions() {
            let dir_str = dir.to_string_lossy();
            for s in sessions {
                if s.output_dir.eq_ignore_ascii_case(&dir_str) || s.folder_path.eq_ignore_ascii_case(&dir_str) {
                    if let Ok(segs) = storage.get_segments(&s.id) {
                        for seg in segs {
                            if !found_any || seg.segment_index > max_idx {
                                max_idx = seg.segment_index;
                            }
                            found_any = true;
                        }
                    }
                }
            }
        }
    }

    if found_any {
        max_idx + 1
    } else {
        1
    }
}