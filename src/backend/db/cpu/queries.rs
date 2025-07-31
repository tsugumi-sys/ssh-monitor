use anyhow::Result;
use rusqlite::Connection;
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct CpuResultRow {
    pub host_id: String,
    pub core_count: u32,
    pub usage_percent: f32,
}

#[derive(Debug, Clone)]
pub struct CpuDetailRow {
    pub host_id: String,
    pub model_name: String,
    pub core_count: u32,
    pub usage_percent: f32,
    pub per_core: Vec<f32>,
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

pub async fn fetch_latest_cpu_by_host(
    conn: &Arc<Mutex<Connection>>,
    host_id: &str,
) -> Result<Option<CpuDetailRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT model_name, core_count, usage_percent, per_core_json \
         FROM cpu_results \
         WHERE host_id = ?1 \
         ORDER BY timestamp DESC \
         LIMIT 1",
    )?;
    let mut rows = stmt.query([host_id])?;
    if let Some(row) = rows.next()? {
        let per_core_json: String = row.get(3)?;
        let per_core: Vec<f32> = serde_json::from_str(&per_core_json).unwrap_or_default();
        Ok(Some(CpuDetailRow {
            host_id: host_id.to_string(),
            model_name: row.get::<_, String>(0)?,
            core_count: row.get::<_, i64>(1)? as u32,
            usage_percent: row.get::<_, f64>(2)? as f32,
            per_core,
        }))
    } else {
        Ok(None)
    }
}

pub async fn fetch_latest_cpu_detail_all(
    conn: &Arc<Mutex<Connection>>,
) -> Result<Vec<CpuDetailRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT c.host_id, c.model_name, c.core_count, c.usage_percent, c.per_core_json \
         FROM cpu_results c \
         JOIN (SELECT host_id, MAX(timestamp) AS max_ts FROM cpu_results GROUP BY host_id) t \
           ON c.host_id = t.host_id AND c.timestamp = t.max_ts",
    )?;
    let rows = stmt.query_map([], |row| {
        let per_core_json: String = row.get(4)?;
        let per_core: Vec<f32> = serde_json::from_str(&per_core_json).unwrap_or_default();
        Ok(CpuDetailRow {
            host_id: row.get::<_, String>(0)?,
            model_name: row.get::<_, String>(1)?,
            core_count: row.get::<_, i64>(2)? as u32,
            usage_percent: row.get::<_, f64>(3)? as f32,
            per_core,
        })
    })?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
