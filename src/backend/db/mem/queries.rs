use anyhow::Result;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MemResultRow {
    pub host_id: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub used_percent: f32,
}

pub async fn fetch_latest_mem_all(conn: &Arc<Mutex<Connection>>) -> Result<Vec<MemResultRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT m.host_id, m.total_mb, m.used_mb, m.free_mb, m.used_percent \
         FROM mem_results m \
         JOIN (SELECT host_id, MAX(timestamp) AS max_ts FROM mem_results GROUP BY host_id) t \
           ON m.host_id = t.host_id AND m.timestamp = t.max_ts",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MemResultRow {
            host_id: row.get::<_, String>(0)?,
            total_mb: row.get::<_, i64>(1)? as u64,
            used_mb: row.get::<_, i64>(2)? as u64,
            free_mb: row.get::<_, i64>(3)? as u64,
            used_percent: row.get::<_, f64>(4)? as f32,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
