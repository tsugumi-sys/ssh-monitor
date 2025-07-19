use anyhow::Result;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct GpuResultRow {
    pub host_id: String,
    pub gpu_index: u32,
    pub name: String,
    pub memory_total_mb: Option<u64>,
    pub memory_used_mb: Option<u64>,
    pub temperature_c: Option<f32>,
    pub raw_output: Option<String>,
}

pub async fn fetch_latest_gpu_all(conn: &Arc<Mutex<Connection>>) -> Result<Vec<GpuResultRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT g.host_id, g.gpu_index, g.name, g.memory_total_mb, g.memory_used_mb, g.temperature_c, g.raw_output \n         FROM gpu_results g \n         JOIN (SELECT host_id, gpu_index, MAX(timestamp) AS max_ts FROM gpu_results GROUP BY host_id, gpu_index) t \n           ON g.host_id = t.host_id AND g.gpu_index = t.gpu_index AND g.timestamp = t.max_ts",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GpuResultRow {
            host_id: row.get::<_, String>(0)?,
            gpu_index: row.get::<_, i64>(1)? as u32,
            name: row.get::<_, String>(2)?,
            memory_total_mb: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
            memory_used_mb: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
            temperature_c: row.get::<_, Option<f64>>(5)?.map(|v| v as f32),
            raw_output: row.get::<_, Option<String>>(6)?,
        })
    })?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
