use rusqlite::{params, Connection, Result};
use std::path::Path;
use super::models::{SegmentRecord, SessionRecord, SessionSummary};

pub fn insert_session(conn: &Connection, session: &SessionRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO sessions (id, start_qpc, end_qpc, qpc_frequency, width, height, fps, output_dir, created_at, export_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            session.id,
            session.start_qpc,
            session.end_qpc,
            session.qpc_frequency,
            session.width,
            session.height,
            session.fps,
            session.output_dir,
            session.created_at,
            session.export_status
        ],
    )?;
    Ok(())
}

pub fn finish_session(conn: &Connection, session_id: &str, end_qpc: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET end_qpc = ?1 WHERE id = ?2",
        params![end_qpc, session_id],
    )?;
    Ok(())
}

pub fn update_export_status(conn: &Connection, session_id: &str, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET export_status = ?1 WHERE id = ?2",
        params![status, session_id],
    )?;
    Ok(())
}

pub fn insert_segment(conn: &Connection, segment: &SegmentRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO segments (session_id, segment_index, file_path, start_qpc, end_qpc, frame_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            segment.session_id,
            segment.segment_index,
            segment.file_path,
            segment.start_qpc,
            segment.end_qpc,
            segment.frame_count
        ],
    )?;
    Ok(())
}

pub fn update_segment_end(conn: &Connection, session_id: &str, segment_index: u32, end_qpc: i64, frame_count: u64) -> Result<()> {
    conn.execute(
        "UPDATE segments SET end_qpc = ?1, frame_count = ?2 WHERE session_id = ?3 AND segment_index = ?4",
        params![end_qpc, frame_count, session_id, segment_index],
    )?;
    Ok(())
}

pub fn delete_session(conn: &mut Connection, session_id: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM clicks WHERE session_id = ?1", params![session_id])?;
    tx.execute("DELETE FROM pan_samples WHERE session_id = ?1", params![session_id])?;
    tx.execute("DELETE FROM segments WHERE session_id = ?1", params![session_id])?;
    tx.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
    tx.commit()?;
    Ok(())
}

pub fn delete_sessions_by_folder(conn: &mut Connection, folder_path: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM clicks WHERE session_id IN (
            SELECT id FROM sessions WHERE output_dir = ?1 OR output_dir LIKE ?1 || '/%' OR output_dir LIKE ?1 || '\\%'
        )",
        params![folder_path],
    )?;
    tx.execute(
        "DELETE FROM pan_samples WHERE session_id IN (
            SELECT id FROM sessions WHERE output_dir = ?1 OR output_dir LIKE ?1 || '/%' OR output_dir LIKE ?1 || '\\%'
        )",
        params![folder_path],
    )?;
    tx.execute(
        "DELETE FROM segments WHERE session_id IN (
            SELECT id FROM sessions WHERE output_dir = ?1 OR output_dir LIKE ?1 || '/%' OR output_dir LIKE ?1 || '\\%'
        )",
        params![folder_path],
    )?;
    tx.execute(
        "DELETE FROM sessions WHERE output_dir = ?1 OR output_dir LIKE ?1 || '/%' OR output_dir LIKE ?1 || '\\%'",
        params![folder_path],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn update_segment_file_path(conn: &Connection, session_id: &str, segment_index: u32, new_file_path: &str) -> Result<()> {
    conn.execute(
        "UPDATE segments SET file_path = ?1 WHERE session_id = ?2 AND segment_index = ?3",
        params![new_file_path, session_id, segment_index],
    )?;
    Ok(())
}

pub fn rename_folder_in_records(conn: &mut Connection, old_dir: &str, new_dir: &str) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE sessions SET output_dir = ?2 || SUBSTR(output_dir, LENGTH(?1) + 1)
         WHERE output_dir = ?1 OR output_dir LIKE ?1 || '/%' OR output_dir LIKE ?1 || '\\%'",
        params![old_dir, new_dir],
    )?;
    tx.execute(
        "UPDATE segments SET file_path = ?2 || SUBSTR(file_path, LENGTH(?1) + 1)
         WHERE file_path = ?1 OR file_path LIKE ?1 || '/%' OR file_path LIKE ?1 || '\\%'",
        params![old_dir, new_dir],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn get_session(conn: &Connection, session_id: &str) -> Result<SessionRecord> {
    conn.query_row(
        "SELECT id, start_qpc, end_qpc, qpc_frequency, width, height, fps, output_dir, created_at, export_status
         FROM sessions WHERE id = ?1",
        params![session_id],
        |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                start_qpc: row.get(1)?,
                end_qpc: row.get(2)?,
                qpc_frequency: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                fps: row.get(6)?,
                output_dir: row.get(7)?,
                created_at: row.get(8)?,
                export_status: row.get(9)?,
            })
        },
    )
}

pub fn get_segments(conn: &Connection, session_id: &str) -> Result<Vec<SegmentRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, segment_index, file_path, start_qpc, end_qpc, frame_count
         FROM segments WHERE session_id = ?1 ORDER BY segment_index ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(SegmentRecord {
            id: Some(row.get(0)?),
            session_id: row.get(1)?,
            segment_index: row.get(2)?,
            file_path: row.get(3)?,
            start_qpc: row.get(4)?,
            end_qpc: row.get(5)?,
            frame_count: row.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn list_sessions(conn: &Connection) -> Result<Vec<SessionSummary>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.created_at, s.start_qpc, s.end_qpc, s.qpc_frequency, s.width, s.height, s.fps, s.output_dir, s.export_status,
                (SELECT COUNT(*) FROM segments WHERE session_id = s.id) as seg_count,
                (SELECT COUNT(*) FROM clicks WHERE session_id = s.id) as click_count,
                (SELECT file_path FROM segments WHERE session_id = s.id ORDER BY segment_index ASC LIMIT 1) as first_seg_path
         FROM sessions s
         ORDER BY s.created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        let session_id: String = row.get(0)?;
        let start_qpc: i64 = row.get(2)?;
        let end_qpc: i64 = row.get(3)?;
        let freq: i64 = row.get(4)?;
        let duration_seconds = if end_qpc > start_qpc && freq > 0 {
            (end_qpc - start_qpc) as f64 / freq as f64
        } else {
            0.0
        };
        let output_dir: String = row.get(8)?;
        let export_status: String = row.get(9)?;
        let first_seg_path_opt: Option<String> = row.get(12)?;

        let first_segment_name = if let Some(ref fsp) = first_seg_path_opt {
            Path::new(fsp)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "segment_0001.mp4".to_string())
        } else {
            "segment_0001.mp4".to_string()
        };

        let seg_stem = first_seg_path_opt
            .as_ref()
            .and_then(|p| Path::new(p).file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "segment_0001".to_string());

        let expected_zoom = format!("{}_zoom.mp4", seg_stem);
        let legacy_export = format!("zentra_export_{}.mp4", session_id);
        let out_p = Path::new(&output_dir);
        let export_file_name = if out_p.join(&expected_zoom).exists() {
            Some(expected_zoom)
        } else if out_p.join(&legacy_export).exists() {
            Some(legacy_export)
        } else if export_status == "exported" {
            Some(expected_zoom)
        } else {
            None
        };

        let last_component = out_p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let is_session_id_subfolder = last_component.len() == 15
            && last_component.chars().take(8).all(|c| c.is_ascii_digit())
            && last_component.chars().nth(8) == Some('_')
            && last_component.chars().skip(9).take(6).all(|c| c.is_ascii_digit());

        let (folder_path, folder_name) = if is_session_id_subfolder {
            let parent = out_p.parent().unwrap_or(out_p);
            (
                parent.to_string_lossy().to_string(),
                parent.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Default".to_string()),
            )
        } else {
            (
                out_p.to_string_lossy().to_string(),
                out_p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Default".to_string()),
            )
        };

        Ok(SessionSummary {
            id: session_id,
            created_at: row.get(1)?,
            duration_seconds,
            width: row.get(5)?,
            height: row.get(6)?,
            fps: row.get(7)?,
            output_dir,
            export_status,
            segment_count: row.get(10)?,
            click_count: row.get(11)?,
            folder_name,
            folder_path,
            first_segment_name,
            export_file_name,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}