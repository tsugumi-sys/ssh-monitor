use crate::backend::db::cpu::queries;
use crate::tui::states_update::StateJob;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Snapshot of CPU usage for a single host
#[derive(Debug, Clone, Default)]
pub struct CpuSnapshot {
    pub core_count: u32,
    pub usage_percent: f32,
}

/// Stores the latest CPU snapshots by host_id
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

    /// Get the latest snapshot for a given host, if present.
    pub async fn get(&self, host_id: &str) -> Option<CpuSnapshot> {
        self.data.read().await.get(host_id).cloned()
    }

    /// Fetch latest CPU rows from DB and update internal map
    pub async fn update_from_db(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let rows = queries::fetch_latest_cpu_all(conn).await?;
        log::info!("Fetched {} CPU rows from DB", rows.len());
        let mut map = self.data.write().await;
        map.clear();
        for row in rows {
            log::debug!(
                "📥 Inserting CPU snapshot: host_id={}, core_count={}, usage_percent={}",
                row.host_id,
                row.core_count,
                row.usage_percent
            );
            map.insert(
                row.host_id,
                CpuSnapshot {
                    core_count: row.core_count,
                    usage_percent: row.usage_percent,
                },
            );
        }
        Ok(())
    }

    pub async fn snapshot_map(&self) -> HashMap<String, CpuSnapshot> {
        self.data.read().await.clone()
    }
}

// ────────────────────────────
// List View Job Kind Enum
// ────────────────────────────

#[derive(Clone, Debug)]
pub enum ListSshJobKind {
    Cpu(Arc<CpuStates>),
}

#[async_trait::async_trait]
impl StateJob for ListSshJobKind {
    fn name(&self) -> &'static str {
        match self {
            ListSshJobKind::Cpu(_) => "cpu",
        }
    }

    async fn update(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        match self {
            ListSshJobKind::Cpu(state) => state.update_from_db(conn).await,
        }
    }
}
