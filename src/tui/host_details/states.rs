use crate::backend::db::cpu::queries as cpu_queries;
use crate::tui::states_update::StateJob;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Default)]
pub struct CpuSnapshot {
    pub model_name: String,
    pub core_count: u32,
    pub usage_percent: f32,
    pub per_core: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct CpuStates {
    data: Arc<RwLock<HashMap<String, CpuSnapshot>>>,
}

impl Default for CpuStates {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuStates {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, host_id: &str) -> Option<CpuSnapshot> {
        self.data.read().await.get(host_id).cloned()
    }

    pub async fn update_from_db(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let rows = cpu_queries::fetch_latest_cpu_all(conn).await?;
        let mut map = self.data.write().await;
        map.clear();
        for row in rows {
            map.insert(
                row.host_id,
                CpuSnapshot {
                    model_name: row.model_name,
                    core_count: row.core_count,
                    usage_percent: row.usage_percent,
                    per_core: row.per_core,
                },
            );
        }
        Ok(())
    }

    pub async fn snapshot_map(&self) -> HashMap<String, CpuSnapshot> {
        self.data.read().await.clone()
    }
}

#[derive(Clone, Debug)]
pub enum HostDetailsJobKind {
    Cpu(Arc<CpuStates>),
}

#[async_trait::async_trait]
impl StateJob for HostDetailsJobKind {
    fn name(&self) -> &'static str {
        match self {
            HostDetailsJobKind::Cpu(_) => "cpu",
        }
    }

    async fn update(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        match self {
            HostDetailsJobKind::Cpu(state) => state.update_from_db(conn).await,
        }
    }
}
