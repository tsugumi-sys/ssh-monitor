use anyhow::Result;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
pub struct CpuResultInsert {
    pub host_id: String,
    pub model_name: String,
    pub core_count: u32,
    pub usage_percent: f32,
    pub per_core: Vec<f32>,
}

pub async fn store_cpu_result(conn: &Arc<Mutex<Connection>>, data: &CpuResultInsert) -> Result<()> {
    let per_core_json = serde_json::to_string(&data.per_core)?;
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO cpu_results (host_id, model_name, core_count, usage_percent, per_core_json)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            data.host_id,
            data.model_name,
            data.core_count as i64,
            data.usage_percent,
            per_core_json
        ],
    )?;
    Ok(())
}
