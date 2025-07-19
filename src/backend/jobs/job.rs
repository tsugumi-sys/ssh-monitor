use super::cpu::CPU_COMMAND;
use super::disk::DISK_COMMAND;
use super::gpu::GPU_COMMAND;
use super::mem::MEM_COMMAND;
use crate::ssh_config::SshHostInfo;
use anyhow::Result;
use rusqlite::Connection;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct JobResult {
    pub job_name: String,
    pub value: Box<dyn Any + Send + Sync>,
}

#[derive(Clone, Debug)]
pub enum JobKind {
    Cpu,
    Mem,
    Disk,
    Gpu,
}

impl JobKind {
    pub fn name(&self) -> &'static str {
        match self {
            JobKind::Cpu => "cpu",
            JobKind::Mem => "mem",
            JobKind::Disk => "disk",
            JobKind::Gpu => "gpu",
        }
    }

    pub fn tag(&self) -> &'static str {
        self.name()
    }

    pub fn command(&self) -> String {
        match self {
            JobKind::Cpu => CPU_COMMAND.to_string(),
            JobKind::Mem => MEM_COMMAND.to_string(),
            JobKind::Disk => DISK_COMMAND.to_string(),
            JobKind::Gpu => GPU_COMMAND.to_string(),
        }
    }

    pub fn parse(&self, output: &str) -> Result<Option<JobResult>> {
        match self {
            JobKind::Cpu => crate::backend::jobs::cpu::parse_cpu(output),
            JobKind::Mem => crate::backend::jobs::mem::parse_mem(output),
            JobKind::Disk => crate::backend::jobs::disk::parse_disk(output),
            JobKind::Gpu => crate::backend::jobs::gpu::parse_gpu(output),
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
                use crate::backend::db::mem::commands::{MemResultInsert, store_mem_result};
                use crate::backend::jobs::mem::MemInfo;
                let mem_info = result
                    .value
                    .downcast_ref::<MemInfo>()
                    .ok_or_else(|| anyhow::anyhow!("Expected MemInfo for JobKind::Mem"))?;
                let insert = MemResultInsert {
                    host_id: host_id.to_string(),
                    total_mb: mem_info.total_mb,
                    used_mb: mem_info.used_mb,
                    free_mb: mem_info.free_mb,
                    used_percent: mem_info.used_percent,
                };
                store_mem_result(conn, &insert).await
            }
            JobKind::Disk => {
                use crate::backend::db::disk::commands::{DiskResultInsert, store_disk_result};
                use crate::backend::jobs::disk::DiskInfo;

                let disk_infos = result
                    .value
                    .downcast_ref::<Vec<DiskInfo>>()
                    .ok_or_else(|| anyhow::anyhow!("Expected Vec<DiskInfo> for JobKind::Disk"))?;

                for info in disk_infos {
                    let insert = DiskResultInsert {
                        host_id: host_id.to_string(),
                        mount_point: info.mount_point.clone(),
                        total_mb: info.total_mb,
                        used_mb: info.used_mb,
                        available_mb: info.available_mb,
                        used_percent: info.used_percent,
                    };
                    store_disk_result(conn, &insert).await?;
                }

                Ok(())
            }
            JobKind::Gpu => {
                use crate::backend::db::gpu::commands::{GpuResultInsert, store_gpu_result};
                use crate::backend::jobs::gpu::GpuInfo;

                let gpu_infos = result
                    .value
                    .downcast_ref::<Vec<GpuInfo>>()
                    .ok_or_else(|| anyhow::anyhow!("Expected Vec<GpuInfo> for JobKind::Gpu"))?;

                for info in gpu_infos {
                    let insert = GpuResultInsert {
                        host_id: host_id.to_string(),
                        gpu_index: info.index,
                        name: info.name.clone(),
                        memory_total_mb: info.memory_total_mb,
                        memory_used_mb: info.memory_used_mb,
                        temperature_c: info.temperature_c,
                        raw_output: info.raw_output.clone(),
                    };
                    store_gpu_result(conn, &insert).await?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct JobGroup {
    pub name: String,
    pub interval: Duration,
    pub host: SshHostInfo,
    pub jobs: Vec<JobKind>,
}
