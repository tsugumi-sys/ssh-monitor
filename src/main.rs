mod backend;
use backend::db::init_db_connection;
use backend::jobs::executor::JobGroupExecutor;
use backend::jobs::job::{JobGroup, JobKind};
mod ssh_config;
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::widgets::TableState;
use ratatui::{DefaultTerminal, Frame, widgets::ScrollbarState};
use rusqlite::Connection;
use ssh_config::{SharedSshHosts, SshHostInfo, load_ssh_configs};
mod tui;
use crate::tui::list_ssh::states::ListSshJobKind;
use crate::tui::states_update::{StatesJobExecutor, StatesJobGroup};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tui::list_ssh::{
    handle_key as handle_list_key, render as render_list,
    states::{CpuStates, DiskStates, MemStates},
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    List,
    Search,
}

#[derive(Debug)]
pub struct App {
    running: bool,
    event_stream: EventStream,
    pub mode: AppMode,
    pub db: Arc<Mutex<Connection>>,
    pub ssh_hosts: SharedSshHosts,
    pub cpu_states: Arc<CpuStates>,
    pub mem_states: Arc<MemStates>,
    pub disk_states: Arc<DiskStates>,
    pub table_state: TableState,
    pub table_height: usize,
    pub selected_id: Option<String>,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub search_query: String,
    pub visible_hosts: Vec<(String, SshHostInfo)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let ssh_hosts = load_ssh_configs().unwrap_or_default(); // now a HashMap
        let mut visible_hosts: Vec<(String, SshHostInfo)> = ssh_hosts
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        visible_hosts.sort_by_key(|(_, h)| h.name.clone());
        let selected_id = visible_hosts.first().map(|(id, _)| id.clone());
        let db = Arc::new(Mutex::new(init_db_connection()));
        let cpu_states = Arc::new(CpuStates::new());
        let mem_states = Arc::new(MemStates::new());
        let disk_states = Arc::new(DiskStates::new());
        Self {
            running: false,
            event_stream: EventStream::new(),
            mode: AppMode::List,
            db,
            ssh_hosts: Arc::new(Mutex::new(ssh_hosts)),
            cpu_states,
            mem_states,
            disk_states,
            // Table
            table_height: 0,
            table_state: TableState::default().with_selected(Some(0)),
            selected_id,
            // Vertical Scrolling
            vertical_scroll: 0,
            vertical_scroll_state: ScrollbarState::new(0),
            // Search
            search_query: String::new(),
            visible_hosts,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let executor = JobGroupExecutor::new(self.db.clone());

        {
            let hosts = self.ssh_hosts.lock().await;
            for (host_id, host) in hosts.iter() {
                let group = JobGroup {
                    name: host_id.clone(),
                    interval: std::time::Duration::from_secs(1),
                    host: host.clone(),
                    jobs: vec![JobKind::Cpu, JobKind::Mem, JobKind::Disk, JobKind::Gpu],
                };

                executor.register_group(group).await;
            }
        }

        executor.run_all().await;
        let _status_executor = self.register_status_update_jobs().await;

        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::List | AppMode::Search => render_list(self, frame),
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                if let Some(Ok(evt)) = event {
                    match evt {
                        Event::Key(key)
                            if key.kind == KeyEventKind::Press
                                => self.on_key_event(key),
                        Event::Mouse(_) => {}
                        Event::Resize(_, _) => {}
                        _ => {}
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Sleep for a short duration to avoid busy waiting.
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match self.mode {
            AppMode::List => match key.code {
                KeyCode::Char('/') => {
                    self.mode = AppMode::Search;
                    self.search_query.clear();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                    handle_list_key(self, key);
                    self.update_selected_id_from_table();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                    handle_list_key(self, key);
                    self.update_selected_id_from_table();
                }
                _ => handle_list_key(self, key),
            },
            AppMode::Search => match key.code {
                KeyCode::Esc => {
                    self.mode = AppMode::List;
                    self.search_query.clear();
                    self.vertical_scroll = 0;
                }
                KeyCode::Enter => {
                    self.mode = AppMode::List;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }

                _ => {}
            },
        }
    }

    pub fn update_selected_id_from_table(&mut self) {
        if let Some(index) = self.table_state.selected() {
            if index < self.visible_hosts.len() {
                let (id, _) = &self.visible_hosts[index];
                self.selected_id = Some(id.clone());
            }
        }
    }

    pub async fn register_job_groups(&self) {
        let executor = JobGroupExecutor::new(self.db.clone());

        let hosts = self.ssh_hosts.lock().await;
        for (host_id, host) in hosts.iter() {
            let group = JobGroup {
                name: host_id.clone(),
                interval: std::time::Duration::from_secs(60),
                host: host.clone(),
                jobs: vec![JobKind::Cpu, JobKind::Mem, JobKind::Disk, JobKind::Gpu],
            };

            executor.register_group(group).await;
        }

        executor.run_all().await;
    }

    pub async fn register_status_update_jobs(&self) {
        let list_executor = StatesJobExecutor::new(self.db.clone());
        let list_job_group = StatesJobGroup {
            name: "list_view".to_string(),
            interval: Duration::from_secs(5),
            jobs: vec![
                ListSshJobKind::Cpu(self.cpu_states.clone()),
                ListSshJobKind::Mem(self.mem_states.clone()),
                ListSshJobKind::Disk(self.disk_states.clone()),
            ],
        };

        list_executor.register_group(list_job_group).await;
        list_executor.run_all().await;
    }
}
