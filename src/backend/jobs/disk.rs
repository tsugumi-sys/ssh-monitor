use super::job::JobResult;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub used_percent: f32,
}

/// `JobKind::Disk` によって呼び出されるパース関数
pub fn parse_disk(output: &str) -> Result<Option<JobResult>> {
    let mut results = vec![];

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let total_mb = parts[1].parse::<u64>().unwrap_or(0);
        let used_mb = parts[2].parse::<u64>().unwrap_or(0);
        let available_mb = parts[3].parse::<u64>().unwrap_or(0);
        let used_percent = parts[4].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
        let mount_point = parts[5].to_string();

        results.push(DiskInfo {
            mount_point,
            total_mb,
            used_mb,
            available_mb,
            used_percent,
        });
    }

    Ok(Some(JobResult {
        job_name: "disk".into(),
        value: Box::new(results),
    }))
}
