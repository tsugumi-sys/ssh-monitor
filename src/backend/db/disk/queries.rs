use anyhow::Result;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct DiskResultRow {
    pub host_id: String,
    pub total_mb: u64,
    pub used_mb: u64,
}

pub async fn fetch_latest_disk_all(conn: &Arc<Mutex<Connection>>) -> Result<Vec<DiskResultRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT d.host_id, SUM(d.total_mb) as total_mb, SUM(d.used_mb) as used_mb \
         FROM disk_results d \
         JOIN (SELECT host_id, MAX(timestamp) AS max_ts FROM disk_results GROUP BY host_id) t \
           ON d.host_id = t.host_id AND d.timestamp = t.max_ts \
         GROUP BY d.host_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DiskResultRow {
            host_id: row.get::<_, String>(0)?,
            total_mb: row.get::<_, i64>(1)? as u64,
            used_mb: row.get::<_, i64>(2)? as u64,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
