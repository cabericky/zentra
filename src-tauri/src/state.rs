use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::capture::CapturePipeline;
use crate::encoder::RollingEncoder;
use crate::live_magnifier::LiveMagnifierController;
use crate::mouse_hook::MouseHookController;
use crate::storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub session_id: Option<String>,
    pub elapsed_seconds: f64,
    pub click_count: usize,
    pub segment_count: usize,
    pub debug_mode: bool,
    pub live_zoom: bool,
    pub live_magnifier: bool,
    pub record_system_audio: bool,
    pub record_mic: bool,
}

pub struct ActiveSession {
    pub session_id: String,
    pub start_qpc: i64,
    pub qpc_frequency: i64,
    pub debug_mode: bool,
    pub live_zoom: bool,
    pub live_magnifier: Option<Arc<LiveMagnifierController>>,
    pub record_system_audio: bool,
    pub record_mic: bool,
    pub capture: CapturePipeline,
    pub encoder: RollingEncoder,
    pub audio: crate::audio::AudioPipeline,
    pub mouse_hook: MouseHookController,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignatedFolderInfo {
    pub current_path: String,
    pub current_name: String,
    pub base_path: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderItem {
    pub name: String,
    pub path: String,
    pub is_active: bool,
    pub exists: bool,
    pub session_count: usize,
}

pub struct AppState {
    pub storage: Arc<Storage>,
    pub active_session: Arc<Mutex<Option<ActiveSession>>>,
    pub base_dir: PathBuf,
    pub output_dir: Arc<Mutex<PathBuf>>,
    pub export_cancel_flag: Arc<AtomicBool>,
}