use crate::backend::db::cpu::queries as cpu_queries;
use crate::backend::db::disk::queries as disk_queries;
use crate::tui::states_update::StateJob;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Default)]
pub struct CpuSnapshot {
    pub core_count: u32,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Default)]
pub struct DiskSnapshot {
    pub total_mb: u64,
    pub used_mb: u64,
    pub used_percent: f32,
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

#[derive(Debug, Clone)]
pub struct DiskStates {
    data: Arc<RwLock<HashMap<String, DiskSnapshot>>>,
}

impl Default for DiskStates {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskStates {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, host_id: &str) -> Option<DiskSnapshot> {
        self.data.read().await.get(host_id).cloned()
    }

    pub async fn update_from_db(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let rows = disk_queries::fetch_latest_disk_all(conn).await?;
        log::info!("Fetched {} disk rows from DB", rows.len());
        let mut map = self.data.write().await;
        map.clear();
        for row in rows {
            let used_percent = if row.total_mb > 0 {
                (row.used_mb as f32 / row.total_mb as f32) * 100.0
            } else {
                0.0
            };
            map.insert(
                row.host_id,
                DiskSnapshot {
                    total_mb: row.total_mb,
                    used_mb: row.used_mb,
                    used_percent,
                },
            );
        }
        Ok(())
    }

    pub async fn snapshot_map(&self) -> HashMap<String, DiskSnapshot> {
        self.data.read().await.clone()
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
    Disk(Arc<DiskStates>),
}

#[async_trait::async_trait]
impl StateJob for ListSshJobKind {
    fn name(&self) -> &'static str {
        match self {
            ListSshJobKind::Cpu(_) => "cpu",
            ListSshJobKind::Disk(_) => "disk",
        }
    }

    async fn update(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        match self {
            ListSshJobKind::Cpu(state) => state.update_from_db(conn).await,
            ListSshJobKind::Disk(state) => state.update_from_db(conn).await,
        }
    }
}
