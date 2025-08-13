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

#[derive(Debug, Clone)]
pub struct DiskVolumeRow {
    pub host_id: String,
    pub mount_point: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub used_percent: f32,
}

pub async fn fetch_latest_disk_all(conn: &Arc<Mutex<Connection>>) -> Result<Vec<DiskResultRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT 
            d.host_id,
            SUM(CASE WHEN d.mount_point = '/' THEN d.total_mb + d.used_mb + d.available_mb ELSE 0 END) AS total_capacity,
            SUM(d.used_mb) AS total_usage
        FROM disk_results d
        JOIN (SELECT host_id, MAX(timestamp) AS max_ts FROM disk_results GROUP BY host_id) t
          ON d.host_id = t.host_id AND d.timestamp = t.max_ts
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

pub async fn fetch_latest_disk_volumes(
    conn: &Arc<Mutex<Connection>>,
    host_id: &str,
) -> Result<Vec<DiskVolumeRow>> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT 
            d.host_id,
            d.mount_point,
            d.total_mb,
            d.used_mb,
            d.available_mb,
            d.used_percent
        FROM disk_results d
        JOIN (SELECT host_id, MAX(timestamp) AS max_ts FROM disk_results WHERE host_id = ?1 GROUP BY host_id) t
          ON d.host_id = t.host_id AND d.timestamp = t.max_ts
        WHERE d.host_id = ?1
        ORDER BY d.mount_point",
    )?;

    let rows = stmt.query_map([host_id], |row| {
        Ok(DiskVolumeRow {
            host_id: row.get::<_, String>(0)?,
            mount_point: row.get::<_, String>(1)?,
            total_mb: row.get::<_, i64>(2)? as u64,
            used_mb: row.get::<_, i64>(3)? as u64,
            available_mb: row.get::<_, i64>(4)? as u64,
            used_percent: row.get::<_, f64>(5)? as f32,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
