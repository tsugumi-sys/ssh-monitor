use anyhow::Result;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
pub struct GpuResultInsert {
    pub host_id: String,
    pub gpu_index: u32,
    pub name: String,
    pub memory_total_mb: Option<u64>,
    pub memory_used_mb: Option<u64>,
    pub temperature_c: Option<f32>,
    pub raw_output: Option<String>,
}

pub async fn store_gpu_result(conn: &Arc<Mutex<Connection>>, data: &GpuResultInsert) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO gpu_results (
            host_id,
            gpu_index,
            name,
            memory_total_mb,
            memory_used_mb,
            temperature_c,
            raw_output
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            data.host_id,
            data.gpu_index as i64,
            data.name,
            data.memory_total_mb.map(|v| v as i64),
            data.memory_used_mb.map(|v| v as i64),
            data.temperature_c,
            data.raw_output,
        ],
    )?;
    Ok(())
}
