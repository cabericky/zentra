pub mod hotkeys;
pub mod models;
pub mod reconciler;
pub mod schema;
pub mod sessions;
pub mod settings;
pub mod telemetry;

pub use hotkeys::*;
pub use models::*;

use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};

pub struct Storage {
    db_path: PathBuf,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let path = db_path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let storage = Self { db_path: path };
        storage.init_tables()?;
        Ok(storage)
    }

    pub fn open_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    pub fn init_tables(&self) -> Result<()> {
        let conn = self.open_connection()?;
        schema::init_tables(&conn)
    }

    // --- Session & Segment Operations (delegated to storage/sessions.rs) ---

    pub fn insert_session(&self, session: &SessionRecord) -> Result<()> {
        let conn = self.open_connection()?;
        sessions::insert_session(&conn, session)
    }

    pub fn finish_session(&self, session_id: &str, end_qpc: i64) -> Result<()> {
        let conn = self.open_connection()?;
        sessions::finish_session(&conn, session_id, end_qpc)
    }

    pub fn update_export_status(&self, session_id: &str, status: &str) -> Result<()> {
        let conn = self.open_connection()?;
        sessions::update_export_status(&conn, session_id, status)
    }

    pub fn insert_segment(&self, segment: &SegmentRecord) -> Result<()> {
        let conn = self.open_connection()?;
        sessions::insert_segment(&conn, segment)
    }

    pub fn update_segment_end(&self, session_id: &str, segment_index: u32, end_qpc: i64, frame_count: u64) -> Result<()> {
        let conn = self.open_connection()?;
        sessions::update_segment_end(&conn, session_id, segment_index, end_qpc, frame_count)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut conn = self.open_connection()?;
        sessions::delete_session(&mut conn, session_id)
    }

    pub fn delete_sessions_by_folder(&self, folder_path: &str) -> Result<()> {
        let mut conn = self.open_connection()?;
        sessions::delete_sessions_by_folder(&mut conn, folder_path)
    }

    pub fn update_segment_file_path(&self, session_id: &str, segment_index: u32, new_file_path: &str) -> Result<()> {
        let conn = self.open_connection()?;
        sessions::update_segment_file_path(&conn, session_id, segment_index, new_file_path)
    }

    pub fn rename_folder_in_records(&self, old_dir: &str, new_dir: &str) -> Result<()> {
        let mut conn = self.open_connection()?;
        sessions::rename_folder_in_records(&mut conn, old_dir, new_dir)
    }

    pub fn get_session(&self, session_id: &str) -> Result<SessionRecord> {
        let conn = self.open_connection()?;
        sessions::get_session(&conn, session_id)
    }

    pub fn get_segments(&self, session_id: &str) -> Result<Vec<SegmentRecord>> {
        let conn = self.open_connection()?;
        sessions::get_segments(&conn, session_id)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let conn = self.open_connection()?;
        sessions::list_sessions(&conn)
    }

    pub fn reconcile_and_list_sessions(&self, base_dir: &Path) -> Result<Vec<SessionSummary>> {
        reconciler::reconcile_and_list_sessions(self, base_dir)
    }

    // --- Telemetry Operations (delegated to storage/telemetry.rs) ---

    pub fn insert_clicks_batch(&self, clicks: &[ClickRecord]) -> Result<()> {
        let mut conn = self.open_connection()?;
        telemetry::insert_clicks_batch(&mut conn, clicks)
    }

    pub fn insert_pan_samples_batch(&self, samples: &[PanSampleRecord]) -> Result<()> {
        let mut conn = self.open_connection()?;
        telemetry::insert_pan_samples_batch(&mut conn, samples)
    }

    pub fn get_clicks(&self, session_id: &str) -> Result<Vec<ClickRecord>> {
        let conn = self.open_connection()?;
        telemetry::get_clicks(&conn, session_id)
    }

    pub fn get_pan_samples(&self, session_id: &str) -> Result<Vec<PanSampleRecord>> {
        let conn = self.open_connection()?;
        telemetry::get_pan_samples(&conn, session_id)
    }

    // --- Settings Operations (delegated to storage/settings.rs) ---

    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.open_connection()?;
        settings::save_setting(&conn, key, value)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.open_connection()?;
        settings::get_setting(&conn, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_lifecycle() {
        let temp_db = std::env::temp_dir().join(format!(
            "test_zentra_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let storage = Storage::new(&temp_db).unwrap();
        let list = storage.list_sessions().unwrap();
        assert_eq!(list.len(), 0);
        let _ = std::fs::remove_file(&temp_db);
    }
}