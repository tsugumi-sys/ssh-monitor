mod ssh_config;
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::widgets::TableState;
use ratatui::{DefaultTerminal, Frame, widgets::ScrollbarState};
use ssh_config::{SharedSshHosts, SshHostInfo, load_ssh_configs};
mod backend;
mod tui;
use std::sync::Arc;
use tokio::sync::Mutex;
use tui::list_ssh::{handle_key as handle_list_key, render as render_list};

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
    pub ssh_hosts: SharedSshHosts,
    pub table_state: TableState,
    pub table_height: usize,
    pub selected_id: Option<String>,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub search_query: String,
    pub visible_hosts: Vec<(String, SshHostInfo)>,
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
        Self {
            running: false,
            event_stream: EventStream::new(),
            mode: AppMode::List,
            ssh_hosts: Arc::new(Mutex::new(ssh_hosts)),
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

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
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
                match event {
                    Some(Ok(evt)) => {
                        match evt {
                            Event::Key(key)
                                if key.kind == KeyEventKind::Press
                                    => self.on_key_event(key),
                            Event::Mouse(_) => {}
                            Event::Resize(_, _) => {}
                            _ => {}
                        }
                    }
                    _ => {}
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
}
