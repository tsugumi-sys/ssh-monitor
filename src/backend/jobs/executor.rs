use crate::backend::jobs::job::{JobGroup, JobKind, JobResult};
use crate::backend::ssh::{connect_ssh_session, run_ssh_command};
use anyhow::Result;
use log::{error, info, warn};
use rusqlite::Connection;
use std::{collections::HashMap, sync::Arc};
use tokio::time::{self, Duration, timeout};
use tokio::{
    sync::{Mutex, RwLock},
    task,
};

#[derive(Clone, Debug)]
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

    #[allow(dead_code)]
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
        let timeout_duration = Duration::from_secs(5);
        match timeout(timeout_duration, run_group_once(group.clone())).await {
            Ok(Ok(results)) => {
                for result in results {
                    if let Some(job_kind) = group.jobs.iter().find(|j| j.name() == result.job_name)
                    {
                        if let Err(e) = job_kind.save(&conn, &group.host.id, &result).await {
                            warn!("❌ Failed to save {} result to DB: {e}", result.job_name);
                        }
                    } else {
                        warn!("❌ Unknown job type: {}", result.job_name);
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("❌ Error running group '{:?}': {:?}", group.name, e);
            }
            Err(e) => {
                warn!("❌ Timeout while running group '{}': {e}", group.name);
            }
        }
        time::sleep(group.interval).await;
    }
}

async fn run_group_once(group: JobGroup) -> Result<Vec<JobResult>> {
    info!("🚀 Running group '{}'", group.name);

    let Some(full_cmd) = build_combined_command(&group.jobs) else {
        warn!("⚠️ No jobs in group '{}'", group.name);
        return Ok(vec![]); // Return empty results instead of raising an error
    };
    info!("📜 Full command to execute:\n{}", full_cmd);

    let session = match connect_ssh_session(&group.host) {
        Ok(session) => session,
        Err(e) => {
            warn!("❌ SSH connection failed for group '{}': {e}", group.name);
            return Ok(vec![]); // Return empty results if connection fails
        }
    };

    let output = match run_ssh_command(&session, &full_cmd) {
        Ok(output) => output,
        Err(e) => {
            warn!("❌ SSH execution failed for group '{}': {e}", group.name);
            return Ok(vec![]); // Return empty results if command execution fails
        }
    };

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
