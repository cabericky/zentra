use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub start_qpc: i64,
    pub end_qpc: i64,
    pub qpc_frequency: i64,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub output_dir: String,
    pub created_at: String,
    pub export_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentRecord {
    pub id: Option<i64>,
    pub session_id: String,
    pub segment_index: u32,
    pub file_path: String,
    pub start_qpc: i64,
    pub end_qpc: i64,
    pub frame_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickRecord {
    pub id: Option<i64>,
    pub session_id: String,
    pub qpc_timestamp: i64,
    pub time_seconds: f64,
    pub x: i32,
    pub y: i32,
    pub button: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanSampleRecord {
    pub id: Option<i64>,
    pub session_id: String,
    pub qpc_timestamp: i64,
    pub time_seconds: f64,
    pub scale: f32,
    pub center_x: f32,
    pub center_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub created_at: String,
    pub duration_seconds: f64,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub segment_count: usize,
    pub click_count: usize,
    pub export_status: String,
    pub output_dir: String,
    pub folder_name: String,
    pub folder_path: String,
    pub first_segment_name: String,
    pub export_file_name: Option<String>,
}