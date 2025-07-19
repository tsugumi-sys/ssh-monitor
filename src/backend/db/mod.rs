use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod mem;

pub fn get_default_db_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "tsugumi-sys", "SshMonitor")
        .expect("❌ Failed to determine data directory");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("❌ Failed to create data directory");
    data_dir.join("ssh_monitor.db")
}

pub fn init_db_connection() -> Connection {
    let db_path = get_default_db_path();

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
        CREATE TABLE IF NOT EXISTS gpu_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host_id TEXT NOT NULL,
            gpu_index INTEGER NOT NULL,
            name TEXT,
            memory_total_mb INTEGER,
            memory_used_mb INTEGER,
            temperature_c REAL,
            raw_output TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )
    .expect("❌ Failed to create gpu_results table");

    conn.execute(
        r#"
        DELETE FROM job_results
        WHERE timestamp < datetime('now', '-1 hour')
        "#,
        [],
    )
    .expect("❌ Failed to delete old job_results");

    for table in ["cpu_results", "mem_results", "disk_results", "gpu_results"] {
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
