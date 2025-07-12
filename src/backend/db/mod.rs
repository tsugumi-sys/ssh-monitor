use crate::backend::jobs::job::JobResult;
use crate::backend::jobs::{cpu::CpuInfo, mem::MemInfo, disk::DiskInfo};
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
        CREATE TABLE IF NOT EXISTS mem_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id TEXT NOT NULL,
            total_mb INTEGER NOT NULL,
            used_mb INTEGER NOT NULL,
            free_mb INTEGER NOT NULL,
            used_percent REAL NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )
    .expect("❌ Failed to create mem_results table");

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS disk_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id TEXT NOT NULL,
            mount_point TEXT NOT NULL,
            total_mb INTEGER NOT NULL,
            used_mb INTEGER NOT NULL,
            available_mb INTEGER NOT NULL,
            used_percent REAL NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )
    .expect("❌ Failed to create disk_results table");

    conn.execute(
        r#"
        DELETE FROM job_results
        WHERE timestamp < datetime('now', '-1 hour')
        "#,
        [],
    )
    .expect("❌ Failed to delete old job_results");

    for table in ["cpu_results", "mem_results", "disk_results"] {
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

pub async fn store_cpu_result(
    conn: &Arc<Mutex<Connection>>,
    host_id: &str,
    info: &CpuInfo,
) -> Result<()> {
    let per_core_json = serde_json::to_string(&info.per_core)?;
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO cpu_results (host_id, model_name, core_count, usage_percent, per_core_json)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![host_id, info.model_name, info.core_count as i64, info.usage_percent, per_core_json],
    )?;
    Ok(())
}

pub async fn store_mem_result(
    conn: &Arc<Mutex<Connection>>,
    host_id: &str,
    info: &MemInfo,
) -> Result<()> {
    let conn = conn.lock().await;
    conn.execute(
        r#"
        INSERT INTO mem_results (host_id, total_mb, used_mb, free_mb, used_percent)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![host_id, info.total_mb as i64, info.used_mb as i64, info.free_mb as i64, info.used_percent],
    )?;
    Ok(())
}

pub async fn store_disk_results(
    conn: &Arc<Mutex<Connection>>,
    host_id: &str,
    disks: &[DiskInfo],
) -> Result<()> {
    let conn = conn.lock().await;
    let tx = conn.transaction()?;
    for d in disks {
        tx.execute(
            r#"
            INSERT INTO disk_results (host_id, mount_point, total_mb, used_mb, available_mb, used_percent)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![host_id, d.mount_point, d.total_mb as i64, d.used_mb as i64, d.available_mb as i64, d.used_percent],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn latest_cpu(conn: &Connection, host_id: &str) -> Result<Option<CpuInfo>> {
    let mut stmt = conn.prepare(
        "SELECT model_name, core_count, usage_percent, per_core_json FROM cpu_results WHERE host_id=?1 ORDER BY timestamp DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(params![host_id])?;
    if let Some(row) = rows.next()? {
        let per_core_json: String = row.get(3)?;
        let per_core: Vec<f32> = serde_json::from_str(&per_core_json).unwrap_or_default();
        Ok(Some(CpuInfo {
            model_name: row.get(0)?,
            core_count: row.get::<_, i64>(1)? as usize,
            usage_percent: row.get(2)?,
            per_core,
        }))
    } else {
        Ok(None)
    }
}

pub fn latest_mem(conn: &Connection, host_id: &str) -> Result<Option<MemInfo>> {
    let mut stmt = conn.prepare(
        "SELECT total_mb, used_mb, free_mb, used_percent FROM mem_results WHERE host_id=?1 ORDER BY timestamp DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(params![host_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(MemInfo {
            total_mb: row.get::<_, i64>(0)? as u64,
            used_mb: row.get::<_, i64>(1)? as u64,
            free_mb: row.get::<_, i64>(2)? as u64,
            used_percent: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn latest_disks(conn: &Connection, host_id: &str) -> Result<Vec<DiskInfo>> {
    let mut stmt = conn.prepare(
        "SELECT mount_point, total_mb, used_mb, available_mb, used_percent FROM disk_results WHERE host_id=?1 AND timestamp = (SELECT MAX(timestamp) FROM disk_results WHERE host_id=?1)",
    )?;
    let rows = stmt.query_map(params![host_id], |row| {
        Ok(DiskInfo {
            mount_point: row.get(0)?,
            total_mb: row.get::<_, i64>(1)? as u64,
            used_mb: row.get::<_, i64>(2)? as u64,
            available_mb: row.get::<_, i64>(3)? as u64,
            used_percent: row.get(4)?,
        })
    })?;
    let mut results = vec![];
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}
