use crate::ssh_config::SshHostInfo;
use anyhow::Result;
use erased_serde::Serialize;
use std::time::Duration;

pub struct JobResult {
    pub job_name: String,
    pub value: Box<dyn Serialize + Send + Sync>,
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
}

#[derive(Clone)]
pub struct JobGroup {
    pub name: String,
    pub interval: Duration,
    pub host: SshHostInfo,
    pub jobs: Vec<JobKind>,
}
