use crate::models::State;
use crate::parser::Parser;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEvent};
use notify::Watcher;
use ratatui::widgets::ListState;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum View {
    Chats,
    Stats,
    Tools,
    Memory,
    Plans,
    Health,
    Timeline,
    Skills,
    MCP,
    Settings,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProjectSort {
    Date,
    Cost,
    Tokens,
    Name,
}

pub struct App {
    pub view: View,
    pub state: Option<State>,
    pub list_state: ListState,
    pub detail_scroll: u16,
    pub is_loading: bool,
    pub should_quit: bool,
    pub search_query: String,
    pub is_searching: bool,
    pub sort_mode: ProjectSort,
    pub last_action_msg: Option<(String, std::time::Instant)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Chats,
            state: None,
            list_state: ListState::default(),
            detail_scroll: 0,
            is_loading: true,
            should_quit: false,
            search_query: String::new(),
            is_searching: false,
            sort_mode: ProjectSort::Date,
            last_action_msg: None,
        }
    }

    pub async fn run<B: ratatui::backend::Backend>(mut self, mut terminal: ratatui::Terminal<B>) -> Result<()> 
    where 
        B::Error: std::error::Error + Send + Sync + 'static 
    {
        let (tx, mut rx) = mpsc::channel(10);
        let (refresh_tx, mut refresh_rx) = mpsc::channel::<()>(1);
        let parser = Parser::new()?;
        let base_dir = parser.base_dir.clone();

        // 1. Setup Watcher
        let notify_tx = refresh_tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if res.is_ok() {
                let _ = notify_tx.blocking_send(());
            }
        })?;

        if base_dir.exists() {
            watcher.watch(&base_dir, notify::RecursiveMode::Recursive)?;
        }

        // 2. Background Sync Task
        tokio::spawn(async move {
            // Initial load
            if let Ok(new_state) = parser.get_full_state() {
                let _ = tx.send(new_state).await;
            }

            // Watch for changes
            while let Some(_) = refresh_rx.recv().await {
                // Debounce: wait a bit for more writes to finish
                tokio::time::sleep(Duration::from_millis(100)).await;
                // Drain any extra signals during sleep
                while let Ok(_) = refresh_rx.try_recv() {}

                if let Ok(new_state) = parser.get_full_state() {
                    let _ = tx.send(new_state).await;
                }
            }
        });

        while !self.should_quit {
            terminal.draw(|f| crate::ui::render(f, &mut self))?;

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key(key);
                        }
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            event::MouseEventKind::ScrollDown => self.detail_scroll = self.detail_scroll.saturating_add(3),
                            event::MouseEventKind::ScrollUp => self.detail_scroll = self.detail_scroll.saturating_sub(3),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Drain all pending updates
            let mut state_updated = false;
            while let Ok(new_state) = rx.try_recv() {
                self.state = Some(new_state);
                state_updated = true;
            }

            if state_updated {
                self.is_loading = false;
                if self.list_state.selected().is_none() {
                    if let Some(s) = &self.state {
                        if !s.all_sessions.is_empty() {
                            self.list_state.select(Some(0));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.is_searching {
            match key.code {
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.search_query.clear();
                }
                KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.list_state.select(Some(0));
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.list_state.select(Some(0));
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('e') => self.export_current_view(),
            KeyCode::Char('s') => {
                self.sort_mode = match self.sort_mode {
                    ProjectSort::Date => ProjectSort::Cost,
                    ProjectSort::Cost => ProjectSort::Tokens,
                    ProjectSort::Tokens => ProjectSort::Name,
                    ProjectSort::Name => ProjectSort::Date,
                };
            }
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            KeyCode::Char('1') => { self.view = View::Chats; self.reset_view(); }
            KeyCode::Char('2') => { self.view = View::Stats; self.reset_view(); }
            KeyCode::Char('3') => { self.view = View::Tools; self.reset_view(); }
            KeyCode::Char('4') => { self.view = View::Memory; self.reset_view(); }
            KeyCode::Char('5') => { self.view = View::Plans; self.reset_view(); }
            KeyCode::Char('6') => { self.view = View::Health; self.reset_view(); }
            KeyCode::Char('7') => { self.view = View::Timeline; self.reset_view(); }
            KeyCode::Char('8') => { self.view = View::Skills; self.reset_view(); }
            KeyCode::Char('9') => { self.view = View::MCP; self.reset_view(); }
            KeyCode::Char('0') => { self.view = View::Settings; self.reset_view(); }
            KeyCode::Down | KeyCode::Char('j') => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) || key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.detail_scroll = self.detail_scroll.saturating_add(1);
                } else {
                    self.move_cursor(1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) || key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                } else {
                    self.move_cursor(-1);
                }
            }
            KeyCode::Char('J') => self.detail_scroll = self.detail_scroll.saturating_add(1),
            KeyCode::Char('K') => self.detail_scroll = self.detail_scroll.saturating_sub(1),
            KeyCode::PageDown => self.detail_scroll = self.detail_scroll.saturating_add(10),
            KeyCode::PageUp => self.detail_scroll = self.detail_scroll.saturating_sub(10),
            _ => {}
        }
    }

    fn reset_view(&mut self) {
        self.list_state.select(Some(0));
        self.detail_scroll = 0;
        self.search_query.clear();
        self.is_searching = false;
    }

    fn move_cursor(&mut self, delta: i32) {
        if let Some(state) = &self.state {
            let count = match self.view {
                View::Chats | View::Tools => {
                    if self.search_query.is_empty() {
                        state.all_sessions.len()
                    } else {
                        state.all_sessions.iter().filter(|s| {
                            format_session_search(s).to_lowercase().contains(&self.search_query.to_lowercase())
                        }).count()
                    }
                },
                View::Stats => state.projects.len() + 1,
                View::Memory => state.projects.iter().map(|p| p.memory_files.len()).sum(),
                View::Plans => state.projects.iter().map(|p| p.plan_files.len()).sum(),
                View::Health => state.health.len(),
                View::Timeline => state.timeline.len(),
                View::Skills => state.skills.len(),
                View::MCP => state.mcp_servers.len(),
                View::Settings => state.settings.as_object().map_or(0, |o| o.len()),
            };
            if count == 0 { return; }
            let current = self.list_state.selected().unwrap_or(0);
            let next = (current as i32 + delta).rem_euclid(count as i32) as usize;
            self.list_state.select(Some(next));
            self.detail_scroll = 0;
        }
    }

    fn export_current_view(&mut self) {
        if let Some(state) = &self.state {
            let selected = self.list_state.selected().unwrap_or(0);
            match self.view {
                View::Chats | View::Tools | View::Timeline => {
                    let filtered: Vec<_> = if self.search_query.is_empty() {
                        state.all_sessions.iter().collect()
                    } else {
                        let query = self.search_query.to_lowercase();
                        state.all_sessions.iter().filter(|s| {
                            crate::ui::components::format_session_search(s).to_lowercase().contains(&query)
                        }).collect()
                    };
                    if let Some(sess) = filtered.get(selected) {
                        if let Ok(json) = serde_json::to_string_pretty(sess) {
                            let filename = format!("geminiscope_session_{}.json", &sess.session_id[..8]);
                            if std::fs::write(&filename, json).is_ok() {
                                self.last_action_msg = Some((format!("󰄬 Exported to {}", filename), std::time::Instant::now()));
                            }
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

fn format_session_search(sess: &crate::models::Session) -> String {
    let mut text = String::new();
    for msg in &sess.messages {
        text.push_str(&format_value(&msg.content));
    }
    text
}

fn format_value(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    format!("{}", content)
}
