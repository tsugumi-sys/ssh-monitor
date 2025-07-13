use crate::backend::jobs::job::JobResult;
use anyhow::Result;
use directories::ProjectDirs;
use rusqlite::{Connection, params};
use serde_json::to_string;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

fn get_default_db_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "tsugumi-sys", "SshMonitor")
        .expect("❌ Failed to determine data directory");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("❌ Failed to create data directory");
    data_dir.join("ssh_monitor.db")
}

pub fn init_db_connection() -> Connection {
    let db_path = get_default_db_path();
    println!("📂 Using database at: {}", db_path.display());

    let conn = Connection::open(&db_path).expect("❌ Failed to open sqlite db");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS job_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id TEXT NOT NULL,
            job_name TEXT NOT NULL,
            value_json TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )
    .expect("❌ Failed to create job_results table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS cpu_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id TEXT NOT NULL,
            model_name TEXT NOT NULL,
            core_count INTEGER NOT NULL,
            usage_percent REAL NOT NULL,
            per_core_json TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )
    .expect("❌ Failed to create cpu_results table");

    conn.execute(
        r#"
        DELETE FROM job_results
        WHERE timestamp < datetime('now', '-1 hour')
        "#,
        [],
    )
    .expect("❌ Failed to delete old job_results");

    for table in ["cpu_results"] {
        conn.execute(
            &format!(
                "DELETE FROM {} WHERE timestamp < datetime('now', '-1 hour')",
                table
            ),
            [],
        )
        .expect("❌ Failed to delete old metrics");
    }

    conn
}

pub async fn store_job_result(
    conn: &Arc<Mutex<Connection>>,
    host_id: &str,
    result: &JobResult,
) -> Result<()> {
    let value_json = to_string(&result.value)?;
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO job_results (host_id, job_name, value_json)
        VALUES (?1, ?2, ?3)
        "#,
        params![host_id, result.job_name, value_json],
    )?;
    Ok(())
}
