use crate::backend::jobs::job::JobResult;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde_json::to_string;

/// SQLite にジョブ結果を保存する
pub async fn store_job_result(host_id: &str, result: &JobResult) -> Result<()> {
    let conn =
        Connection::open("job_results.db").with_context(|| "Failed to open job_results.db")?;

    // テーブルがなければ作成（初回のみ）
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
    )?;

    // value を JSON に変換
    let value_json = to_string(&result.value)?;

    // データ挿入
    conn.execute(
        r#"
        INSERT INTO job_results (host_id, job_name, value_json)
        VALUES (?1, ?2, ?3)
        "#,
        params![host_id, result.job_name, value_json],
    )?;

    Ok(())
}
