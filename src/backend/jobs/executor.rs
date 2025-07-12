use crate::backend::db::store_job_result;
use crate::backend::jobs::job::{JobGroup, JobKind, JobResult};
use crate::backend::ssh::{connect_ssh_session, run_ssh_command};
use anyhow::{Result, anyhow};
use log::{error, info, warn};
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, task, time};

#[derive(Clone, Default)]
pub struct JobGroupExecutor {
    groups: Arc<RwLock<HashMap<String, JobGroup>>>,
}

impl JobGroupExecutor {
    pub async fn run_all(&self) {
        let groups = self.groups.read().await;
        for group in groups.values().cloned() {
            task::spawn(run_group_task(group));
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

async fn run_group_task(group: JobGroup) {
    loop {
        match run_group_once(group.clone()).await {
            Ok(results) => {
                for result in results {
                    if let Err(e) = store_job_result(&group.host.name, &result).await {
                        error!("❌ Failed to save result to DB: {e}");
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

    let session = match connect_ssh_session(&group.host) {
        Ok(s) => s,
        Err(e) => {
            error!("❌ SSH connect error: {}", e);
            return Ok(vec![]);
        }
    };

    let output = match run_ssh_command(&session, &full_cmd) {
        Ok(o) => o,
        Err(e) => {
            error!("❌ SSH exec error: {}", e);
            return Ok(vec![]);
        }
    };
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
    use crate::backend::jobs::executor::JobGroupExecutor;
    use crate::backend::jobs::job::{JobGroup, JobKind};
    use crate::ssh_config::SshHostInfo;
    use std::time::Duration;

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

    let executor = JobGroupExecutor::default();
    executor.register_group(group).await;

    let results = executor.run("test").await.unwrap();
    for r in results {
        println!(
            "✅ {} => {}",
            r.job_name,
            serde_json::to_string_pretty(&r.value).unwrap()
        );
    }
}
