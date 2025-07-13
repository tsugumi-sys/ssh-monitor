use crate::ssh_config::SshHostInfo;
use anyhow::Result;
use erased_serde::Serialize;
use rusqlite::Connection;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct JobResult {
    pub job_name: String,
    pub value: Box<dyn Any + Send + Sync>,
}

#[derive(Clone)]
pub enum JobKind {
    Cpu,
    Mem,
    Disk,
}

impl JobKind {
    pub fn name(&self) -> &'static str {
        match self {
            JobKind::Cpu => "cpu",
            JobKind::Mem => "mem",
            JobKind::Disk => "disk",
        }
    }

    pub fn tag(&self) -> &'static str {
        self.name()
    }

    pub fn command(&self) -> String {
        match self {
            JobKind::Cpu => {
                r#"bash -c '(lscpu && echo __STAT__ && top -bn1 -w 512) || (sysctl -a | grep machdep.cpu && echo __STAT__ && ps -A -o %cpu)'"#.to_string()
            }
            JobKind::Mem => {
                r#"bash -c 'uname -s && echo __MEM__ && (free -m || (echo __MAC__ && sysctl -n hw.memsize && vm_stat))'"#.to_string()
            }
            JobKind::Disk => {
                "df -Pm | tail -n +2".to_string()
            }
        }
    }

    pub fn parse(&self, output: &str) -> Result<Option<JobResult>> {
        match self {
            JobKind::Cpu => crate::backend::jobs::cpu::parse_cpu(output),
            JobKind::Mem => crate::backend::jobs::mem::parse_mem(output),
            JobKind::Disk => crate::backend::jobs::disk::parse_disk(output),
        }
    }

    pub async fn save(
        &self,
        conn: &Arc<Mutex<Connection>>,
        host_id: &str,
        result: &JobResult,
    ) -> Result<()> {
        match self {
            JobKind::Cpu => {
                use crate::backend::db::cpu::commands::{CpuResultInsert, store_cpu_result};
                use crate::backend::jobs::cpu::CpuInfo;

                let cpu_info = result
                    .value
                    .downcast_ref::<CpuInfo>()
                    .ok_or_else(|| anyhow::anyhow!("Expected CpuInfo for JobKind::Cpu"))?;

                let insert = CpuResultInsert {
                    host_id: host_id.to_string(),
                    model_name: cpu_info.model_name.clone(),
                    core_count: cpu_info.core_count as u32,
                    usage_percent: cpu_info.usage_percent,
                    per_core: cpu_info.per_core.clone(),
                };

                store_cpu_result(conn, &insert).await
            }
            JobKind::Mem => {
                // TODO: implement mem insertion logic here
                Ok(())
            }
            JobKind::Disk => {
                // TODO: implement disk insertion logic here
                Ok(())
            }
        }
    }
}

#[derive(Clone)]
pub struct JobGroup {
    pub name: String,
    pub interval: Duration,
    pub host: SshHostInfo,
    pub jobs: Vec<JobKind>,
}
