use super::job::JobResult;
use anyhow::{Result, anyhow};
use serde::Serialize;

pub const MEM_COMMAND: &str = r#"bash -c 'uname -s && echo __MEM__ && (free -m || (echo __MAC__ && sysctl -n hw.memsize && vm_stat))'"#;

#[derive(Debug, Serialize)]
pub struct MemInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub used_percent: f32,
}

pub fn parse_mem(output: &str) -> Result<Option<JobResult>> {
    let mut sections = output.split("__MEM__");
    let platform = sections
        .next()
        .ok_or_else(|| anyhow!("missing platform section"))?
        .trim();
    let mem_output = sections
        .next()
        .ok_or_else(|| anyhow!("missing memory section"))?
        .trim();

    let info_opt = match platform {
        "Linux" => {
            if let Some(line) = mem_output.lines().find(|l| l.contains("Mem:")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let total = parts[1].parse().unwrap_or(0);
                    let used = parts[2].parse().unwrap_or(0);
                    let free = parts.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
                    let percent = if total > 0 {
                        (used as f32 / total as f32) * 100.0
                    } else {
                        0.0
                    };
                    Some(MemInfo {
                        total_mb: total,
                        used_mb: used,
                        free_mb: free,
                        used_percent: percent,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }

        "Darwin" => {
            let mut lines = mem_output.lines();

            let total_bytes = lines
                .find(|line| line.trim().chars().all(|c| c.is_ascii_digit()))
                .ok_or_else(|| anyhow!("missing hw.memsize value"))?
                .trim()
                .parse::<u64>()
                .unwrap_or(0);

            let mut page_size = 4096u64;

            let mut counters = std::collections::HashMap::from([
                ("Pages active", 0u64),
                ("Pages speculative", 0u64),
                ("Pages occupied by compressor", 0u64),
                ("Pages wired down", 0u64),
                ("Pages inactive", 0u64),
            ]);

            for line in lines {
                if line.contains("page size of") {
                    if let Some(v) = line
                        .split("page size of")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                    {
                        page_size = v.parse().unwrap_or(4096);
                    }
                    continue;
                }

                if let Some((key, value)) = line.split_once(':') {
                    let val = value.trim().trim_end_matches('.').replace('.', "");
                    if let Ok(count) = val.parse::<u64>() {
                        if let Some(entry) = counters.get_mut(key.trim()) {
                            *entry = count;
                        }
                    }
                }
            }

            let used_pages: u64 = counters.values().sum();
            let used_mb = (used_pages * page_size) / 1024 / 1024;
            let total_mb = total_bytes / 1024 / 1024;
            let free_mb = total_mb.saturating_sub(used_mb);
            let percent = if total_mb > 0 {
                (used_mb as f32 / total_mb as f32) * 100.0
            } else {
                0.0
            };

            Some(MemInfo {
                total_mb,
                used_mb,
                free_mb,
                used_percent: percent,
            })
        }

        _ => None,
    };

    Ok(info_opt.map(|info| JobResult {
        job_name: "mem".into(),
        value: Box::new(info),
    }))
}
