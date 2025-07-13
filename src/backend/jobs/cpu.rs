use super::job::JobResult;
use anyhow::Result;
use serde::Serialize;

pub const CPU_COMMAND: &str = r#"bash -c '(lscpu && echo __STAT__ && top -bn1 -w 512) || (sysctl -a | grep machdep.cpu && echo __STAT__ && ps -A -o %cpu)'"#;

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub model_name: String,
    pub core_count: usize,
    pub usage_percent: f32,
    pub per_core: Vec<f32>,
}

pub fn parse_cpu(output: &str) -> Result<Option<JobResult>> {
    let parts: Vec<&str> = output.split("__STAT__").collect();
    if parts.len() != 2 {
        return Ok(None);
    }

    let (info_part, stat_part) = (parts[0], parts[1]);
    let is_mac = info_part.contains("machdep.cpu");

    let (model_name, core_count) = if is_mac {
        let name = info_part
            .lines()
            .find(|l| l.contains("machdep.cpu.brand_string"))
            .and_then(|l| l.split(':').nth(1))
            .unwrap_or("")
            .trim()
            .to_string();

        let cores = info_part
            .lines()
            .find(|l| l.contains("machdep.cpu.core_count"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1);
        (name, cores)
    } else {
        let name = info_part
            .lines()
            .find(|l| l.contains("Model name"))
            .or_else(|| info_part.lines().find(|l| l.contains("model name")))
            .and_then(|l| l.split(':').nth(1))
            .unwrap_or("")
            .trim()
            .to_string();

        let cores = info_part
            .lines()
            .find(|l| l.contains("CPU(s):"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1);
        (name, cores)
    };

    let (usage_percent, per_core) = if is_mac {
        let mut sum = 0f32;
        let mut count = 0;
        for line in stat_part.lines() {
            if let Ok(p) = line.trim().parse::<f32>() {
                sum += p;
                count += 1;
            }
        }
        (sum, vec![])
    } else {
        let cpu_line = stat_part
            .lines()
            .find(|l| l.contains("Cpu(s):"))
            .unwrap_or("");

        let usage = cpu_line
            .split(',')
            .find(|s| s.contains("us"))
            .and_then(|s| s.split('%').next())
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0.0);

        (usage, vec![])
    };

    let info = CpuInfo {
        model_name,
        core_count,
        usage_percent,
        per_core,
    };

    Ok(Some(JobResult {
        job_name: "cpu".into(),
        value: Box::new(info),
    }))
}
