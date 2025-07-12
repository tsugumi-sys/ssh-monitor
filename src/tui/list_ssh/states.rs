use std::collections::HashMap;
use rusqlite::Connection;

use crate::backend::db::{latest_cpu, latest_disks, latest_mem};
use crate::backend::jobs::{cpu::CpuInfo, mem::MemInfo, disk::DiskInfo};
use crate::ssh_config::SshHostInfo;

#[derive(Debug, Clone)]
pub struct DiskOverview {
    pub total_mb: u64,
    pub used_mb: u64,
    pub used_percent: f32,
}

#[derive(Debug, Clone)]
pub struct HostMetrics {
    pub cpu: Option<CpuInfo>,
    pub mem: Option<MemInfo>,
    pub disk: Option<DiskOverview>,
}

#[derive(Default, Debug)]
pub struct ListStates {
    pub metrics: HashMap<String, HostMetrics>,
}

impl ListStates {
    pub fn refresh(&mut self, conn: &Connection, hosts: &[(String, SshHostInfo)]) {
        for (id, _info) in hosts {
            let cpu = latest_cpu(conn, id).ok().flatten();
            let mem = latest_mem(conn, id).ok().flatten();
            let disks = latest_disks(conn, id).ok().unwrap_or_default();
            let disk = summarize_disks(&disks);
            self.metrics.insert(
                id.clone(),
                HostMetrics { cpu, mem, disk },
            );
        }
    }

    pub fn get(&self, id: &str) -> Option<&HostMetrics> {
        self.metrics.get(id)
    }
}

fn summarize_disks(disks: &[DiskInfo]) -> Option<DiskOverview> {
    if disks.is_empty() {
        return None;
    }
    let total: u64 = disks.iter().map(|d| d.total_mb).sum();
    let used: u64 = disks.iter().map(|d| d.used_mb).sum();
    if total == 0 {
        return None;
    }
    let used_percent = used as f32 / total as f32 * 100.0;
    Some(DiskOverview {
        total_mb: total,
        used_mb: used,
        used_percent,
    })
}
