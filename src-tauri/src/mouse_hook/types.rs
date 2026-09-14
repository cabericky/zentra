use windows::Win32::Foundation::POINT;

/// A recorded click event.
#[derive(Debug, Clone)]
pub struct ClickEvent {
    pub session_id: String,
    pub qpc_timestamp: i64,
    pub time_seconds: f64,
    pub x: i32,
    pub y: i32,
    pub button: String,
}

pub struct RawClickEvent {
    pub qpc: i64,
    pub point: POINT,
    pub button: &'static str,
}