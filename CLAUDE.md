# SSH Monitor - Claude Code Instructions

## Adding New Database States

This document outlines the steps to add new database states to the SSH Monitor application.

### Steps to Add New States

1. **Add Database Query Method**
   - Location: `src/backend/db/{module}/queries.rs`
   - Create a new struct for the query result (e.g., `CpuTimelineRow`)
   - Implement the database query function (e.g., `fetch_cpu_usage_timeline`)
   - Follow existing patterns for async functions and error handling

2. **Create Timeline States Structure**
   - Location: `src/tui/host_details/states.rs`
   - Create snapshot struct (e.g., `CpuTimelineSnapshot`) with `#[derive(Debug, Clone, Default)]`
   - Create states struct (e.g., `CpuTimelineStates`) with `Arc<RwLock<SnapshotType>>`
   - Implement `Default`, `new()`, `get()`, and `update_from_db()` methods
   - Use proper error handling with `Result<(), Box<dyn std::error::Error + Send + Sync>>`

3. **Update Host Details State**
   - Location: `src/tui/host_details/states.rs`
   - Add new field to `HostDetailsState` struct
   - Update the `new()` method to initialize the new states

4. **Add Job Kind Variant**
   - Location: `src/tui/host_details/states.rs`
   - Add new variant to `DetailsJobKind` enum
   - Update `StateJob` implementation:
     - Add case in `name()` method
     - Add case in `update()` method with proper error conversion

5. **Register Job in Main App**
   - Location: `src/main.rs`
   - Add new job to the `details_job_group` in `register_status_update_jobs()`
   - Follow the pattern: `DetailsJobKind::NewKind(self.details_states.new_field.clone())`

### Example Implementation

For CPU Timeline states, the implementation included:

- Database: `CpuTimelineRow` and `fetch_cpu_usage_timeline()`
- States: `CpuTimelineSnapshot` and `CpuTimelineStates`
- Job: `DetailsJobKind::CpuTimeline` variant
- Integration: Added to details job group with 5-second interval

### Key Patterns

- All states use `Arc<RwLock<>>` for thread-safe access
- Database queries return `Result<Vec<RowType>>`
- States update methods are async and use the database connection
- Jobs run on configurable intervals (typically 5 seconds for UI updates)
- Error handling uses `anyhow::Result` for consistency