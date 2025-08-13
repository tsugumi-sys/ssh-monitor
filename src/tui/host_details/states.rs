use crate::backend::db::cpu::queries as cpu_queries;
use crate::backend::db::disk::queries as disk_queries;
use crate::backend::db::mem::queries as mem_queries;
use crate::tui::states_update::StateJob;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Default)]
pub struct CpuDetailSnapshot {
    pub model_name: String,
    pub core_count: u32,
    pub usage_percent: f32,
    pub per_core: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct MemDetailSnapshot {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub used_percent: f32,
}

#[derive(Debug, Clone, Default)]
pub struct DiskDetailSnapshot {
    pub total_mb: u64,
    pub used_mb: u64,
    pub used_percent: f32,
}

#[derive(Debug, Clone)]
pub struct DiskVolumeSnapshot {
    pub mount_point: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub used_percent: f32,
}

#[derive(Debug, Clone)]
pub struct CpuDetailStates {
    data: Arc<RwLock<HashMap<String, CpuDetailSnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct MemDetailStates {
    data: Arc<RwLock<HashMap<String, MemDetailSnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct DiskDetailStates {
    data: Arc<RwLock<HashMap<String, DiskDetailSnapshot>>>,
}

impl Default for CpuDetailStates {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuDetailStates {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, host_id: &str) -> Option<CpuDetailSnapshot> {
        self.data.read().await.get(host_id).cloned()
    }

    pub async fn update_from_db(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let rows = cpu_queries::fetch_latest_cpu_detail_all(conn).await?;
        let mut map = self.data.write().await;
        map.clear();
        for row in rows {
            map.insert(
                row.host_id,
                CpuDetailSnapshot {
                    model_name: row.model_name,
                    core_count: row.core_count,
                    usage_percent: row.usage_percent,
                    per_core: row.per_core,
                },
            );
        }
        Ok(())
    }

    pub async fn snapshot_map(&self) -> HashMap<String, CpuDetailSnapshot> {
        self.data.read().await.clone()
    }
}

impl Default for MemDetailStates {
    fn default() -> Self {
        Self::new()
    }
}

impl MemDetailStates {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, host_id: &str) -> Option<MemDetailSnapshot> {
        self.data.read().await.get(host_id).cloned()
    }

    pub async fn update_from_db(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let rows = mem_queries::fetch_latest_mem_all(conn).await?;
        let mut map = self.data.write().await;
        map.clear();
        for row in rows {
            map.insert(
                row.host_id,
                MemDetailSnapshot {
                    total_mb: row.total_mb,
                    used_mb: row.used_mb,
                    free_mb: row.free_mb,
                    used_percent: row.used_percent,
                },
            );
        }
        Ok(())
    }

    pub async fn snapshot_map(&self) -> HashMap<String, MemDetailSnapshot> {
        self.data.read().await.clone()
    }
}
impl Default for DiskDetailStates {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskDetailStates {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, host_id: &str) -> Option<DiskDetailSnapshot> {
        self.data.read().await.get(host_id).cloned()
    }

    pub async fn get_volumes(
        &self,
        host_id: &str,
        conn: &Arc<Mutex<Connection>>,
    ) -> Result<Vec<DiskVolumeSnapshot>> {
        let rows = disk_queries::fetch_latest_disk_volumes(conn, host_id).await?;
        let volumes: Vec<DiskVolumeSnapshot> = rows
            .into_iter()
            .map(|row| DiskVolumeSnapshot {
                mount_point: row.mount_point,
                total_mb: row.total_mb,
                used_mb: row.used_mb,
                available_mb: row.available_mb,
                used_percent: row.used_percent,
            })
            .collect();
        Ok(volumes)
    }

    pub async fn update_from_db(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let rows = disk_queries::fetch_latest_disk_all(conn).await?;
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
                DiskDetailSnapshot {
                    total_mb: row.total_mb,
                    used_mb: row.used_mb,
                    used_percent,
                },
            );
        }
        Ok(())
    }

    pub async fn snapshot_map(&self) -> HashMap<String, DiskDetailSnapshot> {
        self.data.read().await.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct HostDetailsState {
    pub cpu: Arc<CpuDetailStates>,
    pub mem: Arc<MemDetailStates>,
    pub disk: Arc<DiskDetailStates>,
}

impl HostDetailsState {
    pub fn new() -> Self {
        Self {
            cpu: Arc::new(CpuDetailStates::new()),
            mem: Arc::new(MemDetailStates::new()),
            disk: Arc::new(DiskDetailStates::new()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DetailsJobKind {
    Cpu(Arc<CpuDetailStates>),
    Mem(Arc<MemDetailStates>),
    Disk(Arc<DiskDetailStates>),
}

#[async_trait::async_trait]
impl StateJob for DetailsJobKind {
    fn name(&self) -> &'static str {
        match self {
            DetailsJobKind::Cpu(_) => "cpu_detail",
            DetailsJobKind::Mem(_) => "mem_detail",
            DetailsJobKind::Disk(_) => "disk_detail",
        }
    }

    async fn update(&self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        match self {
            DetailsJobKind::Cpu(state) => state.update_from_db(conn).await,
            DetailsJobKind::Mem(state) => state.update_from_db(conn).await,
            DetailsJobKind::Disk(state) => state.update_from_db(conn).await,
        }
    }
}
