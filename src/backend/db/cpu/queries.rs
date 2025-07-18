use anyhow::Result;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct CpuResultRow {
    pub host_id: String,
    pub core_count: u32,
    pub usage_percent: f32,
}

pub async fn fetch_latest_cpu_all(conn: &Arc<Mutex<Connection>>) -> Result<Vec<CpuResultRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT c.host_id, c.core_count, c.usage_percent \
         FROM cpu_results c \
         JOIN (SELECT host_id, MAX(timestamp) AS max_ts FROM cpu_results GROUP BY host_id) t \
           ON c.host_id = t.host_id AND c.timestamp = t.max_ts",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CpuResultRow {
            host_id: row.get::<_, String>(0)?,
            core_count: row.get::<_, i64>(1)? as u32,
            usage_percent: row.get::<_, f64>(2)? as f32,
        })
    })?;
    let mut results = vec![];
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
