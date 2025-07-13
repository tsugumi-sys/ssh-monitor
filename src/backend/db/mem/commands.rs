use anyhow::Result;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
pub struct MemResultInsert {
    pub host_id: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub used_percent: f32,
}

pub async fn store_mem_result(conn: &Arc<Mutex<Connection>>, data: &MemResultInsert) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO mem_results (host_id, total_mb, used_mb, free_mb, used_percent)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            data.host_id,
            data.total_mb,
            data.used_mb,
            data.free_mb,
            data.used_percent,
        ],
    )?;
    Ok(())
}
