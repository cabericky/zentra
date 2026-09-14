use rusqlite::{Connection, Result};

/// Initializes the database schema and indexes for Zentra.
pub fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            start_qpc INTEGER NOT NULL,
            end_qpc INTEGER NOT NULL DEFAULT 0,
            qpc_frequency INTEGER NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            fps INTEGER NOT NULL,
            output_dir TEXT NOT NULL,
            created_at TEXT NOT NULL,
            export_status TEXT NOT NULL DEFAULT 'recorded'
        );

        CREATE TABLE IF NOT EXISTS segments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            segment_index INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            start_qpc INTEGER NOT NULL,
            end_qpc INTEGER NOT NULL DEFAULT 0,
            frame_count INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS clicks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            qpc_timestamp INTEGER NOT NULL,
            time_seconds REAL NOT NULL,
            x INTEGER NOT NULL,
            y INTEGER NOT NULL,
            button TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS pan_samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            qpc_timestamp INTEGER NOT NULL,
            time_seconds REAL NOT NULL,
            scale REAL NOT NULL,
            center_x REAL NOT NULL,
            center_y REAL NOT NULL,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE INDEX IF NOT EXISTS idx_clicks_session ON clicks(session_id, qpc_timestamp);
        CREATE INDEX IF NOT EXISTS idx_segments_session ON segments(session_id, segment_index);
        CREATE INDEX IF NOT EXISTS idx_pan_samples_session ON pan_samples(session_id, qpc_timestamp);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "
    )?;
    Ok(())
}