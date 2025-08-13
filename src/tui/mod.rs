pub mod host_details;
pub mod list_ssh;
pub mod states_update;

#[tokio::test]
async fn test_cpu_job_executor_runs() {
    use super::tui::states_update::{StatesJobExecutor, StatesJobGroup};
    use crate::backend::db::cpu::commands::{CpuResultInsert, store_cpu_result};
    use crate::backend::db::init_db_connection;
    use crate::tui::list_ssh::states::{CpuStates, ListSshJobKind};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;

    let db = Arc::new(Mutex::new(init_db_connection()));
    let cpu_states = CpuStates::new();
    let job = ListSshJobKind::Cpu(cpu_states.clone().into());

    {
        let insert = CpuResultInsert {
            host_id: "test-host".to_string(),
            model_name: "FakeCPU".to_string(),
            core_count: 8,
            usage_percent: 33.3,
            per_core: vec![10.0, 20.0, 30.0, 40.0, 10.0, 20.0, 30.0, 40.0],
        };
        store_cpu_result(&db, &insert).await.unwrap();
    }

    let job_group = StatesJobGroup {
        name: "list_view".into(),
        interval: Duration::from_millis(100),
        jobs: vec![job],
    };

    let executor = StatesJobExecutor::new(db.clone());
    executor.register_group(job_group).await;
    executor.run_all().await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let snapshot = cpu_states.get("test-host").await;
    assert!(snapshot.is_some());

    let snapshot = snapshot.unwrap();
    assert_eq!(snapshot.core_count, 8);
    assert!((snapshot.usage_percent - 33.3).abs() < f32::EPSILON);
}
