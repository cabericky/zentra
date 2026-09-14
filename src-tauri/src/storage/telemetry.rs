use rusqlite::{params, Connection, Result};
use super::models::{ClickRecord, PanSampleRecord};

pub fn insert_clicks_batch(conn: &mut Connection, clicks: &[ClickRecord]) -> Result<()> {
    if clicks.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO clicks (session_id, qpc_timestamp, time_seconds, x, y, button)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for c in clicks {
            stmt.execute(params![
                c.session_id,
                c.qpc_timestamp,
                c.time_seconds,
                c.x,
                c.y,
                c.button
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn insert_pan_samples_batch(conn: &mut Connection, samples: &[PanSampleRecord]) -> Result<()> {
    if samples.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO pan_samples (session_id, qpc_timestamp, time_seconds, scale, center_x, center_y)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for s in samples {
            stmt.execute(params![
                s.session_id,
                s.qpc_timestamp,
                s.time_seconds,
                s.scale,
                s.center_x,
                s.center_y
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_clicks(conn: &Connection, session_id: &str) -> Result<Vec<ClickRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, qpc_timestamp, time_seconds, x, y, button
         FROM clicks WHERE session_id = ?1 ORDER BY qpc_timestamp ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(ClickRecord {
            id: Some(row.get(0)?),
            session_id: row.get(1)?,
            qpc_timestamp: row.get(2)?,
            time_seconds: row.get(3)?,
            x: row.get(4)?,
            y: row.get(5)?,
            button: row.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_pan_samples(conn: &Connection, session_id: &str) -> Result<Vec<PanSampleRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, qpc_timestamp, time_seconds, scale, center_x, center_y
         FROM pan_samples WHERE session_id = ?1 ORDER BY qpc_timestamp ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(PanSampleRecord {
            id: Some(row.get(0)?),
            session_id: row.get(1)?,
            qpc_timestamp: row.get(2)?,
            time_seconds: row.get(3)?,
            scale: row.get(4)?,
            center_x: row.get(5)?,
            center_y: row.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}