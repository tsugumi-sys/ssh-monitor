use anyhow::Result;
use rusqlite::Connection;
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
pub struct DiskResultInsert {
    pub host_id: String,
    pub mount_point: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub used_percent: f32,
}

pub async fn store_disk_result(
    conn: &Arc<Mutex<Connection>>,
    data: &DiskResultInsert,
) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO disk_results (
            host_id,
            mount_point,
            total_mb,
            used_mb,
            available_mb,
            used_percent
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            data.host_id,
            data.mount_point,
            data.total_mb,
            data.used_mb,
            data.available_mb,
            data.used_percent,
        ],
    )?;
    Ok(())
}
