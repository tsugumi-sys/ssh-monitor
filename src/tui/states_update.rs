use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::{task, time};

#[async_trait::async_trait]
pub trait StateJob: Send + Sync {
    /// Name of the job (e.g. "cpu", "mem")
    fn name(&self) -> &'static str;

    /// Performs the update from DB or other source
    async fn update(&self, db: &Arc<Mutex<Connection>>) -> Result<()>;
}

#[derive(Clone, Debug)]
pub struct StatesJobGroup<T: StateJob> {
    pub name: String,
    pub interval: std::time::Duration,
    pub jobs: Vec<T>,
}

#[derive(Debug)]
pub struct StatesJobExecutor<T: StateJob> {
    groups: Arc<RwLock<HashMap<String, StatesJobGroup<T>>>>,
    db: Arc<Mutex<Connection>>,
}

impl<T: StateJob + 'static + Clone> StatesJobExecutor<T> {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            db,
        }
    }

    pub async fn register_group(&self, group: StatesJobGroup<T>) {
        let mut groups = self.groups.write().await;
        groups.insert(group.name.clone(), group);
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
}

async fn run_group_task<T: StateJob + 'static + Clone>(
    group: StatesJobGroup<T>,
    conn: Arc<Mutex<Connection>>,
) {
    loop {
        if let Err(e) = run_group_once(&group, &conn).await {
            log::error!("❌ Error running group '{}': {e}", group.name);
        }
        time::sleep(group.interval).await;
    }
}

async fn run_group_once<T: StateJob + 'static + Clone>(
    group: &StatesJobGroup<T>,
    conn: &Arc<Mutex<Connection>>,
) -> Result<()> {
    log::info!("🚀 Running states job group '{}'", group.name);
    for job in &group.jobs {
        if let Err(e) = job.update(conn).await {
            log::error!("❌ Failed to update '{}': {e}", job.name());
        } else {
            log::debug!("✅ Successfully updated '{}'", job.name());
        }
    }
    Ok(())
}
