use crate::backend::jobs::job::{JobGroup, JobKind, JobResult};
use crate::backend::ssh::{connect_ssh_session, run_ssh_command};
use anyhow::{Result, anyhow};
use log::{error, info, warn};
use rusqlite::Connection;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{Mutex, RwLock},
    task, time,
};

#[derive(Clone)]
pub struct JobGroupExecutor {
    groups: Arc<RwLock<HashMap<String, JobGroup>>>,
    db: Arc<Mutex<Connection>>,
}

impl JobGroupExecutor {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            db,
        }
    }

    pub async fn run_all(&self) {
        let groups = self.groups.read().await;
        for group in groups.values().cloned() {
            let db = self.db.clone();
            task::spawn(async move {
                run_group_task(group, db).await;
            });
        }
    }

    pub async fn register_group(&self, group: JobGroup) {
        let mut groups = self.groups.write().await;
        groups.insert(group.name.clone(), group);
    }

    pub async fn run(&self, name: &str) -> Result<Vec<JobResult>> {
        let groups = self.groups.read().await;
        if let Some(group) = groups.get(name) {
            run_group_once(group.clone()).await
        } else {
            Ok(vec![])
        }
    }
}

async fn run_group_task(group: JobGroup, conn: Arc<Mutex<Connection>>) {
    loop {
        match run_group_once(group.clone()).await {
            Ok(results) => {
                for result in results {
                    // Find the corresponding JobKind for this result
                    if let Some(job_kind) = group.jobs.iter().find(|j| j.name() == result.job_name)
                    {
                        if let Err(e) = job_kind.save(&conn, &group.host.name, &result).await {
                            error!("❌ Failed to save {} result to DB: {e}", result.job_name);
                        }
                    } else {
                        error!("❌ Unknown job type: {}", result.job_name);
                    }
                }
            }
            Err(e) => {
                error!("❌ Error running group '{}': {e}", group.name);
            }
        }
        time::sleep(group.interval).await;
    }
}

async fn run_group_once(group: JobGroup) -> Result<Vec<JobResult>> {
    info!("🚀 Running group '{}'", group.name);

    let Some(full_cmd) = build_combined_command(&group.jobs) else {
        warn!("⚠️ No jobs in group '{}'", group.name);
        return Ok(vec![]);
    };
    info!("📜 Full command to execute:\n{}", full_cmd);

    let session =
        connect_ssh_session(&group.host).map_err(|e| anyhow!("SSH connect error: {}", e))?;

    let output =
        run_ssh_command(&session, &full_cmd).map_err(|e| anyhow!("SSH exec error: {}", e))?;

    info!("🖨️ SSH Output:\n{}", output);

    Ok(parse_group_results(&group, &output))
}

fn build_combined_command(jobs: &[JobKind]) -> Option<String> {
    if jobs.is_empty() {
        return None;
    }
    let mut script = String::new();
    for job in jobs {
        script.push_str(&format!("echo __BEGIN_{}__\n", job.tag()));
        script.push_str(&format!("{}\n", job.command()));
        script.push_str(&format!("echo __END_{}__\n", job.tag()));
    }
    Some(script)
}

fn parse_group_results(group: &JobGroup, output: &str) -> Vec<JobResult> {
    let mut results = vec![];
    for job in &group.jobs {
        info!("🔍 Checking job '{}'", job.name());
        if let Some(tagged_output) = extract_tagged_output(output, job.tag()) {
            match job.parse(tagged_output) {
                Ok(Some(mut result)) => {
                    result.job_name = job.name().to_string();
                    info!("✅ Parsed result for '{}'", job.name());
                    results.push(result);
                }
                Ok(None) => {
                    warn!("⚠️ No result parsed for job: {}", job.name());
                }
                Err(err) => {
                    error!("❌ Parse error ({}): {:#}", job.name(), err);
                }
            }
        } else {
            warn!("⛔ No tagged output found for '{}'", job.name());
        }
    }
    results
}

fn extract_tagged_output<'a>(output: &'a str, tag: &str) -> Option<&'a str> {
    let start_tag = format!("__BEGIN_{}__", tag);
    let end_tag = format!("__END_{}__", tag);
    let start = output.find(&start_tag)? + start_tag.len();
    let end = output[start..].find(&end_tag)? + start;
    Some(output[start..end].trim())
}

#[tokio::test]
async fn test_job_executor() {
    let _ = env_logger::builder().is_test(true).try_init();

    use crate::backend::db::init_db_connection;
    use crate::backend::jobs::cpu::CpuInfo;
    use crate::backend::jobs::disk::DiskInfo;
    use crate::backend::jobs::executor::JobGroupExecutor;
    use crate::backend::jobs::job::{JobGroup, JobKind};
    use crate::backend::jobs::mem::MemInfo;
    use crate::ssh_config::SshHostInfo;
    use std::{sync::Arc, time::Duration};
    use tokio::sync::Mutex;

    let db_conn = Arc::new(Mutex::new(init_db_connection()));

    let host = SshHostInfo {
        id: "test".to_string(),
        name: "localtest".to_string(),
        ip: "localhost".to_string(),
        port: 22,
        user: "akira.noda".to_string(),
        identity_file: format!("{}/.ssh/id_ed25519", std::env::var("HOME").unwrap()),
    };

    let group = JobGroup {
        name: "test".to_string(),
        interval: Duration::from_secs(60),
        host,
        jobs: vec![JobKind::Cpu, JobKind::Mem, JobKind::Disk],
    };

    let executor = JobGroupExecutor::new(db_conn.clone());

    executor.register_group(group).await;

    let results = executor.run("test").await.unwrap();

    for r in results {
        match r.job_name.as_str() {
            "cpu" => {
                if let Some(cpu_info) = r.value.downcast_ref::<CpuInfo>() {
                    println!("✅ {} => {:?}", r.job_name, cpu_info);
                } else {
                    println!("✅ {} => (failed to downcast)", r.job_name);
                }
            }
            "mem" => {
                if let Some(mem_info) = r.value.downcast_ref::<MemInfo>() {
                    println!("✅ {} => {:?}", r.job_name, mem_info);
                } else {
                    println!("✅ {} => (failed to downcast)", r.job_name);
                }
            }
            "disk" => {
                if let Some(disk_info) = r.value.downcast_ref::<Vec<DiskInfo>>() {
                    println!("✅ {} => {:?}", r.job_name, disk_info);
                } else {
                    println!("✅ {} => (failed to downcast)", r.job_name);
                }
            }
            _ => {
                println!("✅ {} => {:?}", r.job_name, r.value);
            }
        }
    }
}
