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
    
    // Settings Editing
    pub is_editing_setting: bool,
    pub edit_input: String,
    pub setting_path: Vec<String>,
    pub is_showing_help: bool,
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
            is_editing_setting: false,
            edit_input: String::new(),
            setting_path: Vec::new(),
            is_showing_help: false,
        }
    }

    pub async fn run<B: ratatui::backend::Backend>(mut self, mut terminal: ratatui::Terminal<B>) -> Result<()> 
    where 
        B::Error: std::error::Error + Send + Sync + 'static 
    {
        let (tx, mut rx) = mpsc::channel(10);
        let (refresh_tx, mut refresh_rx) = mpsc::channel::<()>(1);
        let (save_tx, mut save_rx) = mpsc::channel::<serde_json::Value>(1);
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
        let p_save = Parser::new()?;
        let p_refresh = Parser::new()?;
        tokio::spawn(async move {
            // Initial load
            if let Ok(new_state) = p_refresh.get_full_state() {
                let _ = tx.send(new_state).await;
            }

            loop {
                tokio::select! {
                    Some(_) = refresh_rx.recv() => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        while let Ok(_) = refresh_rx.try_recv() {}
                        if let Ok(new_state) = p_refresh.get_full_state() {
                            let _ = tx.send(new_state).await;
                        }
                    }
                    Some(settings) = save_rx.recv() => {
                        let _ = p_save.save_settings(&settings);
                        // Trigger immediate refresh after save
                        if let Ok(new_state) = p_refresh.get_full_state() {
                            let _ = tx.send(new_state).await;
                        }
                    }
                    else => break,
                }
            }
        });

        while !self.should_quit {
            terminal.draw(|f| crate::ui::render(f, &mut self))?;

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key(key, &save_tx);
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

    fn handle_key(&mut self, key: KeyEvent, save_tx: &mpsc::Sender<serde_json::Value>) {
        if self.is_showing_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('q') => self.is_showing_help = false,
                _ => {}
            }
            return;
        }

        if self.is_editing_setting {
            match key.code {
                KeyCode::Esc => self.is_editing_setting = false,
                KeyCode::Enter => {
                    self.commit_setting_edit(save_tx);
                    self.is_editing_setting = false;
                }
                KeyCode::Char(c) => self.edit_input.push(c),
                KeyCode::Backspace => { self.edit_input.pop(); }
                _ => {}
            }
            return;
        }

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
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::Char('?') | KeyCode::Char('h') => self.is_showing_help = true,
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
            KeyCode::Enter => {
                if self.view == View::Settings {
                    self.start_setting_edit(save_tx);
                }
            }
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
                View::Settings => {
                    let mut count = 0;
                    let mut last_section = String::new();
                    for (path, _, _) in crate::ui::infrastructure::get_all_settings(state) {
                        let section = path.split('.').next().unwrap_or("").to_uppercase();
                        if section != last_section {
                            count += 1; // Section header
                            last_section = section;
                        }
                        count += 1; // Setting item
                    }
                    count
                },
            };
            if count == 0 { return; }
            let current = self.list_state.selected().unwrap_or(0);
            let mut next = (current as i32 + delta).rem_euclid(count as i32) as usize;
            
            // Skip over non-selectable headers in settings view
            if self.view == View::Settings {
                if crate::ui::infrastructure::get_setting_at_index(state, next).is_none() {
                    next = (next as i32 + delta.signum()).rem_euclid(count as i32) as usize;
                }
            }

            self.list_state.select(Some(next));
            self.detail_scroll = 0;
        }
    }

    fn start_setting_edit(&mut self, save_tx: &mpsc::Sender<serde_json::Value>) {
        if let Some(state) = &self.state {
            let selected = self.list_state.selected().unwrap_or(0);
            if let Some((path, val, _)) = crate::ui::infrastructure::get_setting_at_index(state, selected) {
                self.setting_path = path.split('.').map(|s| s.to_string()).collect();
                match val {
                    serde_json::Value::Bool(b) => {
                        let mut new_settings = state.settings.clone();
                        self.set_nested_value(&mut new_settings, &self.setting_path, serde_json::Value::Bool(!b));
                        let _ = save_tx.try_send(new_settings);
                        self.last_action_msg = Some((format!("󰄬 Toggled {} to {}", path, !b), std::time::Instant::now()));
                    },
                    serde_json::Value::String(s) => {
                        self.is_editing_setting = true;
                        self.edit_input = s.clone();
                    },
                    serde_json::Value::Number(n) => {
                        self.is_editing_setting = true;
                        self.edit_input = n.to_string();
                    },
                    _ => {}
                }
            }
        }
    }

    fn set_nested_value(&self, root: &mut serde_json::Value, path: &[String], val: serde_json::Value) {
        if !root.is_object() {
            *root = serde_json::json!({});
        }
        let mut curr = root;
        for i in 0..path.len() {
            let key = &path[i];
            if i == path.len() - 1 {
                curr[key.clone()] = val;
                return;
            }
            if !curr.get(key).map_or(false, |v| v.is_object()) {
                curr[key.clone()] = serde_json::json!({});
            }
            curr = curr.get_mut(key).unwrap();
        }
    }

    fn commit_setting_edit(&mut self, save_tx: &mpsc::Sender<serde_json::Value>) {
        if let Some(state) = &mut self.state {
            let mut settings = state.settings.clone();
            
            // Try to parse number if it looks like one
            let new_val = if let Ok(n) = self.edit_input.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else if let Ok(f) = self.edit_input.parse::<f64>() {
                serde_json::Value::from(f)
            } else {
                serde_json::Value::String(self.edit_input.clone())
            };

            self.set_nested_value(&mut settings, &self.setting_path, new_val);
            let _ = save_tx.try_send(settings);
            self.last_action_msg = Some((format!("󰄬 Updated {}", self.setting_path.last().unwrap_or(&"setting".to_string())), std::time::Instant::now()));
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
