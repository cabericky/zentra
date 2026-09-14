use std::path::{Path, PathBuf};
use crate::state::FolderItem;
use crate::storage::Storage;

pub fn record_known_folder(storage: &Storage, folder: &Path) {
    let folder_str = folder.to_string_lossy().to_string();
    let mut known: Vec<String> = if let Ok(Some(saved)) = storage.get_setting("known_folders") {
        serde_json::from_str(&saved).unwrap_or_default()
    } else {
        Vec::new()
    };
    if !known.iter().any(|k| k.eq_ignore_ascii_case(&folder_str)) {
        known.push(folder_str);
        if let Ok(json) = serde_json::to_string(&known) {
            let _ = storage.save_setting("known_folders", &json);
        }
    }
}

pub fn remove_known_folder(storage: &Storage, folder_str: &str) {
    if let Ok(Some(saved)) = storage.get_setting("known_folders") {
        if let Ok(mut known) = serde_json::from_str::<Vec<String>>(&saved) {
            known.retain(|k| !k.eq_ignore_ascii_case(folder_str));
            if let Ok(json) = serde_json::to_string(&known) {
                let _ = storage.save_setting("known_folders", &json);
            }
        }
    }
}

pub fn update_known_folder(storage: &Storage, old_path: &str, new_path: &str) {
    if let Ok(Some(saved)) = storage.get_setting("known_folders") {
        if let Ok(mut known) = serde_json::from_str::<Vec<String>>(&saved) {
            for k in &mut known {
                if k.eq_ignore_ascii_case(old_path) {
                    *k = new_path.to_string();
                }
            }
            if let Ok(json) = serde_json::to_string(&known) {
                let _ = storage.save_setting("known_folders", &json);
            }
        }
    }
}

pub fn list_available_folders(
    storage: &Storage,
    base_dir: &Path,
    current_dir: &Path,
) -> Result<Vec<FolderItem>, String> {
    let mut folders_map: std::collections::BTreeMap<String, FolderItem> = std::collections::BTreeMap::new();

    // 1. Root / Default base folder
    let base_name = base_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Zentra".to_string());
    let base_path_str = base_dir.to_string_lossy().to_string();
    folders_map.insert(
        base_path_str.clone(),
        FolderItem {
            name: format!("{} (Default)", base_name),
            path: base_path_str.clone(),
            is_active: *current_dir == *base_dir,
            exists: base_dir.exists() && base_dir.is_dir(),
            session_count: 0,
        },
    );

    // 2. Discover subdirectories in base_dir
    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let fname = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                // Check if directory name is an auto-generated session folder (e.g. 20260911_120000)
                let is_session_dir = fname.len() == 15
                    && fname.chars().take(8).all(|c| c.is_ascii_digit())
                    && fname.chars().nth(8) == Some('_')
                    && fname.chars().skip(9).take(6).all(|c| c.is_ascii_digit());

                if !is_session_dir {
                    let path_str = path.to_string_lossy().to_string();
                    let is_active = *current_dir == path;
                    folders_map.insert(
                        path_str.clone(),
                        FolderItem {
                            name: fname,
                            path: path_str,
                            is_active,
                            exists: true,
                            session_count: 0,
                        },
                    );
                }
            }
        }
    }

    // 3. Known custom folders from settings
    if let Ok(Some(saved)) = storage.get_setting("known_folders") {
        if let Ok(known) = serde_json::from_str::<Vec<String>>(&saved) {
            for k in known {
                let p = PathBuf::from(&k);
                let fname = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| k.clone());
                let exists = p.exists() && p.is_dir();
                let is_active = *current_dir == p;
                folders_map.entry(k.clone()).or_insert_with(|| FolderItem {
                    name: fname,
                    path: k,
                    is_active,
                    exists,
                    session_count: 0,
                });
            }
        }
    }

    // 4. Ensure current active folder is included even if newly set
    let cur_str = current_dir.to_string_lossy().to_string();
    if !folders_map.contains_key(&cur_str) {
        let cur_name = current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| cur_str.clone());
        folders_map.insert(
            cur_str.clone(),
            FolderItem {
                name: cur_name,
                path: cur_str,
                is_active: true,
                exists: current_dir.exists() && current_dir.is_dir(),
                session_count: 0,
            },
        );
    }

    // 5. Gather existing sessions and count sessions per folder
    if let Ok(sessions) = storage.list_sessions() {
        for s in sessions {
            let s_folder_path = s.folder_path;
            if let Some(item) = folders_map.get_mut(&s_folder_path) {
                item.session_count += 1;
            } else {
                let p = PathBuf::from(&s_folder_path);
                let exists = p.exists() && p.is_dir();
                let is_active = *current_dir == p;
                folders_map.insert(
                    s_folder_path.clone(),
                    FolderItem {
                        name: s.folder_name,
                        path: s_folder_path,
                        is_active,
                        exists,
                        session_count: 1,
                    },
                );
            }
        }
    }

    // Update active flag accurately
    let cur_path_str = current_dir.to_string_lossy().to_string();
    for item in folders_map.values_mut() {
        item.is_active = item.path.eq_ignore_ascii_case(&cur_path_str);
    }

    let mut result: Vec<FolderItem> = folders_map.into_values().collect();
    result.sort_by(|a, b| {
        if a.path.eq_ignore_ascii_case(&base_path_str) {
            std::cmp::Ordering::Less
        } else if b.path.eq_ignore_ascii_case(&base_path_str) {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(result)
}

pub fn create_folder(base_dir: &Path, name_or_path: &str) -> Result<PathBuf, String> {
    let trimmed = name_or_path.trim();
    if trimmed.is_empty() {
        return Err("Folder name cannot be empty.".to_string());
    }

    let p = if Path::new(trimmed).is_absolute() {
        PathBuf::from(trimmed)
    } else {
        if trimmed.contains("..") || trimmed.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*')) {
            return Err("Folder name contains invalid characters.".to_string());
        }
        base_dir.join(trimmed)
    };

    std::fs::create_dir_all(&p).map_err(|e| format!("Failed to create folder: {:?}", e))?;
    Ok(p)
}

pub fn rename_folder(
    storage: &Storage,
    base_dir: &Path,
    old_path_str: &str,
    new_name_str: &str,
) -> Result<PathBuf, String> {
    let trimmed_name = new_name_str.trim();
    if trimmed_name.is_empty() {
        return Err("New folder name cannot be empty.".to_string());
    }
    if trimmed_name.contains('/')
        || trimmed_name.contains('\\')
        || trimmed_name.chars().any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    {
        return Err("Folder name contains invalid characters.".to_string());
    }

    let old_p = PathBuf::from(old_path_str.trim());
    if !old_p.exists() || !old_p.is_dir() {
        return Err(format!("Folder does not exist: {}", old_p.display()));
    }

    if old_p == *base_dir {
        return Err("The default Zentra root folder cannot be renamed.".to_string());
    }

    let parent = old_p.parent().unwrap_or(base_dir);
    let new_p = parent.join(trimmed_name);

    if new_p.exists() {
        return Err(format!("A folder named '{}' already exists.", trimmed_name));
    }

    std::fs::rename(&old_p, &new_p).map_err(|e| format!("Failed to rename folder on disk: {:?}", e))?;

    let old_str = old_p.to_string_lossy().to_string();
    let new_str = new_p.to_string_lossy().to_string();

    let _ = storage.rename_folder_in_records(&old_str, &new_str);
    update_known_folder(storage, &old_str, &new_str);

    Ok(new_p)
}

pub fn delete_folder(
    storage: &Storage,
    base_dir: &Path,
    path_str: &str,
) -> Result<(), String> {
    let p = PathBuf::from(path_str.trim());
    if p == *base_dir {
        return Err("The default Zentra root folder cannot be deleted.".to_string());
    }

    let path_normalized = p.to_string_lossy().to_string();
    let _ = storage.delete_sessions_by_folder(&path_normalized);

    if p.exists() {
        std::fs::remove_dir_all(&p).map_err(|e| format!("Failed to delete folder from disk: {:?}", e))?;
    }

    remove_known_folder(storage, &path_normalized);
    Ok(())
}

pub fn browse_for_folder() -> Result<Option<PathBuf>, String> {
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    };

    unsafe {
        let com_init = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);

        let dialog: IFileOpenDialog = match CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL) {
            Ok(d) => d,
            Err(e) => {
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return Err(format!("Failed to create IFileOpenDialog: {:?}", e));
            }
        };

        let options = match dialog.GetOptions() {
            Ok(opts) => opts,
            Err(e) => {
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return Err(format!("Failed to get dialog options: {:?}", e));
            }
        };

        let _ = dialog.SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM);
        let title = windows::core::w!("Select Designated Recordings Folder");
        let _ = dialog.SetTitle(title);

        if dialog.Show(None).is_err() {
            // User cancelled or closed the dialog
            if com_init.is_ok() {
                CoUninitialize();
            }
            return Ok(None);
        }

        let item = match dialog.GetResult() {
            Ok(it) => it,
            Err(e) => {
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return Err(format!("Failed to get dialog result: {:?}", e));
            }
        };

        let display_name = match item.GetDisplayName(SIGDN_FILESYSPATH) {
            Ok(name) => name,
            Err(e) => {
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return Err(format!("Failed to get folder path: {:?}", e));
            }
        };

        let path_str = match display_name.to_string() {
            Ok(s) => s,
            Err(e) => {
                CoTaskMemFree(Some(display_name.as_ptr() as *const _));
                if com_init.is_ok() {
                    CoUninitialize();
                }
                return Err(format!("Invalid path string encoding: {:?}", e));
            }
        };

        CoTaskMemFree(Some(display_name.as_ptr() as *const _));
        if com_init.is_ok() {
            CoUninitialize();
        }

        let p = PathBuf::from(path_str);
        if !p.exists() || !p.is_dir() {
            return Err(format!("Selected path is not a valid folder: {}", p.display()));
        }

        Ok(Some(p))
    }
}

pub fn open_folder_in_explorer(folder: &Path) -> Result<(), String> {
    if !folder.exists() {
        let _ = std::fs::create_dir_all(folder);
    }
    std::process::Command::new("explorer.exe")
        .arg(folder)
        .spawn()
        .map_err(|e| format!("Failed to open folder in explorer: {:?}", e))?;
    Ok(())
}