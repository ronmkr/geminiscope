use crate::models::{State, View, ProjectSort};
use crate::parser::Parser;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEvent};
use notify::Watcher;
use ratatui::widgets::ListState;
use std::time::Duration;
use tokio::sync::mpsc;

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
    pub is_redacting: bool,

    // Diff State
    pub diff_target: Option<String>, // session_id
    pub diff_results: Option<(String, String, String)>, // (id1, id2, diff_text)
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
            is_redacting: true,
            diff_target: None,
            diff_results: None,
        }
    }

    pub async fn run<B: ratatui::backend::Backend>(mut self, mut terminal: ratatui::Terminal<B>) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        let (save_tx, mut save_rx) = mpsc::channel::<serde_json::Value>(10);
        let (refresh_tx, mut refresh_rx) = mpsc::channel(1);
        
        let parser = Parser::new()?;
        let parser_arc = std::sync::Arc::new(parser);
        let parser_clone = parser_arc.clone();

        // Background worker for settings saving
        tokio::spawn(async move {
            while let Some(settings) = save_rx.recv().await {
                let _ = Parser::save_settings(&settings);
            }
        });

        // Watcher for live updates
        let refresh_tx_watcher = refresh_tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if res.is_ok() {
                let _ = refresh_tx_watcher.try_send(());
            }
        })?;
        watcher.watch(&parser_arc.base_dir, notify::RecursiveMode::Recursive)?;

        // Initial load
        let initial_tx = tx.clone();
        let initial_parser = parser_arc.clone();
        tokio::spawn(async move {
            if let Ok(state) = initial_parser.get_full_state() {
                let _ = initial_tx.send(state).await;
            }
        });

        let mut ticker = tokio::time::interval(Duration::from_millis(100));

        loop {
            if self.should_quit { break; }

            terminal.draw(|f| crate::ui::render(f, &mut self))
                .map_err(|e| anyhow::anyhow!("Terminal error: {e}"))?;

            tokio::select! {
                _ = ticker.tick() => {
                    if let Ok(()) = refresh_rx.try_recv() {
                        let tx = tx.clone();
                        let p = parser_clone.clone();
                        tokio::spawn(async move {
                            if let Ok(state) = p.get_full_state() {
                                let _ = tx.send(state).await;
                            }
                        });
                        // Drain all pending refresh signals
                        while refresh_rx.try_recv().is_ok() {}
                    }
                }
                Some(new_state) = rx.recv() => {
                    self.state = Some(new_state);
                    self.is_loading = false;
                    if self.list_state.selected().is_none()
                        && let Some(s) = &self.state
                            && !s.all_sessions.is_empty() {
                                self.list_state.select(Some(0));
                            }
                }
                res = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(10))) => {
                    if let Ok(Ok(true)) = res {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                self.handle_key(key, &save_tx);
                            }
                        } else if let Event::Mouse(mouse) = event::read()? {
                            self.handle_mouse(mouse);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_mouse(&mut self, mouse: event::MouseEvent) {
        match mouse.kind {
            event::MouseEventKind::ScrollDown => self.detail_scroll = self.detail_scroll.saturating_add(3),
            event::MouseEventKind::ScrollUp => self.detail_scroll = self.detail_scroll.saturating_sub(3),
            event::MouseEventKind::Down(event::MouseButton::Left) => {
                self.handle_rail_click(mouse.row, mouse.column);
            }
            _ => {}
        }
    }

    fn handle_rail_click(&mut self, row: u16, col: u16) {
        if col < 6 {
            let y = i32::from(row);
            if y >= 1 {
                let icon_idx = (y - 1) / 2;
                let all_views = View::all();
                if let Some(&new_view) = all_views.get(icon_idx as usize)
                    && self.view != new_view {
                        self.view = new_view;
                        self.reset_view();
                    }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent, save_tx: &mpsc::Sender<serde_json::Value>) {
        if self.is_showing_help {
            self.handle_help_key(key);
            return;
        }
        if self.is_editing_setting {
            self.handle_edit_key(key, save_tx);
            return;
        }
        if self.is_searching {
            self.handle_search_key(key);
            return;
        }
        self.handle_main_key(key, save_tx);
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?' | 'h' | 'q') => self.is_showing_help = false,
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent, save_tx: &mpsc::Sender<serde_json::Value>) {
        match key.code {
            KeyCode::Esc => self.is_editing_setting = false,
            KeyCode::Enter => {
                self.commit_setting_edit(save_tx);
                self.is_editing_setting = false;
            }
            KeyCode::Char(c) => {
                if self.edit_input.len() < 256 {
                    self.edit_input.push(c);
                }
            }
            KeyCode::Backspace => { self.edit_input.pop(); }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.is_searching = false;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.is_searching = false;
            }
            KeyCode::Char(c) => {
                if self.search_query.len() < 256 {
                    self.search_query.push(c);
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.list_state.select(Some(0));
            }
            _ => {}
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent, save_tx: &mpsc::Sender<serde_json::Value>) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.handle_esc(),
            KeyCode::Char('?' | 'h') => self.is_showing_help = true,
            KeyCode::Char('d') => self.handle_diff_command(),
            KeyCode::Char('o') => self.handle_open_command(),
            KeyCode::Char('r') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.toggle_redaction(),
            KeyCode::Char('e') => self.export_current_view(),
            KeyCode::Char('s') => self.cycle_sort(),
            KeyCode::Char('/') => self.start_search(),
            KeyCode::Char(c) if c.is_ascii_digit() => self.switch_view_by_digit(c),
            KeyCode::Enter => self.handle_enter(save_tx),
            KeyCode::Down | KeyCode::Char('j') => self.handle_down(key.modifiers),
            KeyCode::Up | KeyCode::Char('k') => self.handle_up(key.modifiers),
            KeyCode::Char('J') => self.detail_scroll = self.detail_scroll.saturating_add(1),
            KeyCode::Char('K') => self.detail_scroll = self.detail_scroll.saturating_sub(1),
            KeyCode::PageDown => self.detail_scroll = self.detail_scroll.saturating_add(10),
            KeyCode::PageUp => self.detail_scroll = self.detail_scroll.saturating_sub(10),
            _ => {}
        }
    }

    fn handle_esc(&mut self) {
        if !self.search_query.is_empty() {
            self.search_query.clear();
            self.list_state.select(Some(0));
        } else if self.diff_target.is_some() {
            self.diff_target = None;
            self.last_action_msg = Some(("󰄬 Diff selection cleared".to_string(), std::time::Instant::now()));
        } else if self.view == View::Diff {
            self.view = View::Chats;
            self.reset_view();
        }
    }

    fn toggle_redaction(&mut self) {
        self.is_redacting = !self.is_redacting;
        let status = if self.is_redacting { "Enabled" } else { "Disabled" };
        self.last_action_msg = Some((format!("󰒐 Secret Redaction: {status}"), std::time::Instant::now()));
    }


    fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            ProjectSort::Date => ProjectSort::Cost,
            ProjectSort::Cost => ProjectSort::Tokens,
            ProjectSort::Tokens => ProjectSort::Name,
            ProjectSort::Name => ProjectSort::Date,
        };
    }

    fn start_search(&mut self) {
        self.is_searching = true;
        self.search_query.clear();
    }

    fn switch_view_by_digit(&mut self, c: char) {
        let all_views = View::all();
        let digit = c.to_digit(10).unwrap_or(0) as usize;
        let idx = if digit == 0 { 9 } else { digit - 1 };
        if let Some(&new_view) = all_views.get(idx) {
            self.view = new_view;
            self.reset_view();
        }
    }

    fn handle_enter(&mut self, save_tx: &mpsc::Sender<serde_json::Value>) {
        if self.view == View::Settings {
            self.start_setting_edit(save_tx);
        }
    }

    fn handle_down(&mut self, modifiers: crossterm::event::KeyModifiers) {
        if modifiers.contains(crossterm::event::KeyModifiers::ALT) || modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            self.detail_scroll = self.detail_scroll.saturating_add(1);
        } else {
            self.move_cursor(1);
        }
    }

    fn handle_up(&mut self, modifiers: crossterm::event::KeyModifiers) {
        if modifiers.contains(crossterm::event::KeyModifiers::ALT) || modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            self.detail_scroll = self.detail_scroll.saturating_sub(1);
        } else {
            self.move_cursor(-1);
        }
    }

    fn reset_view(&mut self) {
        self.list_state.select(Some(0));
        self.detail_scroll = 0;
        self.search_query.clear();
        self.is_searching = false;
    }

    fn move_cursor(&mut self, delta: i32) {
        let Some(state) = &self.state else { return };
        let handler = crate::ui::handlers::get_handler(self.view);
        let count = handler.count(state, &self.search_query);
        
        if count == 0 { return; }
        let current = self.list_state.selected().unwrap_or(0);
        let mut next = (current as i32 + delta).rem_euclid(count as i32) as usize;
        
        if self.view == View::Settings && crate::ui::infrastructure::get_setting_at_index(state, next).is_none() {
            next = (next as i32 + delta.signum()).rem_euclid(count as i32) as usize;
        }

        self.list_state.select(Some(next));
        self.detail_scroll = 0;
    }

    fn start_setting_edit(&mut self, save_tx: &mpsc::Sender<serde_json::Value>) {
        if let Some(state) = &self.state {
            let selected = self.list_state.selected().unwrap_or(0);
            if let Some((path, val, _)) = crate::ui::infrastructure::get_setting_at_index(state, selected) {
                self.setting_path = path.split('.').map(std::string::ToString::to_string).collect();
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
            if !curr.get(key).is_some_and(serde_json::Value::is_object) {
                curr[key.clone()] = serde_json::json!({});
            }
            if let Some(next) = curr.get_mut(key) {
                curr = next;
            } else {
                return;
            }
        }
    }

    fn commit_setting_edit(&mut self, save_tx: &mpsc::Sender<serde_json::Value>) {
        if let Some(state) = &mut self.state {
            let mut settings = state.settings.clone();
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

    fn handle_open_command(&mut self) {
        if let Some(state) = &self.state {
            let selected = self.list_state.selected().unwrap_or(0);
            let path = match self.view {
                View::Memory => {
                    let mut all_files = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    for p in &state.projects { 
                        for f in &p.memory_files { 
                            if !seen.contains(&f.path) {
                                all_files.push(&f.path); 
                                seen.insert(f.path.clone());
                            }
                        } 
                    }
                    all_files.get(selected).copied()
                },
                View::Plans => {
                    let mut all_files = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    for p in &state.projects { 
                        for f in &p.plan_files { 
                            if !seen.contains(&f.path) {
                                all_files.push(&f.path); 
                                seen.insert(f.path.clone());
                            }
                        } 
                    }
                    all_files.get(selected).copied()
                },
                _ => return,
            };

            if let Some(file_path) = path {
                let editor = state.settings.get("general")
                    .and_then(|g| g.get("preferredEditor"))
                    .and_then(|e| e.as_str())
                    .unwrap_or("vim");
                
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
                let res = crate::utils::open_in_editor(editor, file_path);
                let _ = crossterm::terminal::enable_raw_mode();
                let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture);

                if let Err(err) = res {
                    self.last_action_msg = Some((format!("⚠ Error opening editor: {err}"), std::time::Instant::now()));
                }
            }
        }
    }

    fn handle_diff_command(&mut self) {
        if let Some(state) = &self.state {
            let selected = self.list_state.selected().unwrap_or(0);
            let sessions = match self.view {
                View::Chats | View::Tools | View::Timeline => state.filtered_sessions(&self.search_query),
                _ => return,
            };

            if let Some(sess) = sessions.get(selected) {
                let current_id = sess.session_id.clone();
                if let Some(target_id) = &self.diff_target {
                    if target_id == &current_id {
                        self.last_action_msg = Some(("⚠ Cannot diff a session with itself".to_string(), std::time::Instant::now()));
                        return;
                    }
                    if let (Some(s1), Some(s2)) = (state.all_sessions.iter().find(|s| &s.session_id == target_id), Some(*sess)) {
                        self.diff_results = Some((target_id.clone(), current_id.clone(), self.generate_diff(s1, s2)));
                        self.view = View::Diff;
                        self.diff_target = None;
                        self.last_action_msg = Some(("󰒺 Generated Diff".to_string(), std::time::Instant::now()));
                    }
                } else {
                    self.diff_target = Some(current_id);
                    self.last_action_msg = Some(("󰄬 Marked for comparison. Select another and press 'd'.".to_string(), std::time::Instant::now()));
                }
            }
        }
    }

    fn generate_diff(&self, s1: &crate::models::Session, s2: &crate::models::Session) -> String {
        use similar::{ChangeTag, TextDiff};
        let t1 = s1.full_text();
        let t2 = s2.full_text();
        let diff = TextDiff::from_lines(&t1, &t2);
        let mut result = String::new();
        result.push_str(&format!("# Diff: {} vs {}\n\n", &s1.session_id[..8], &s2.session_id[..8]));
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push_str(&format!("{sign}{change}"));
        }
        result
    }

    fn export_current_view(&mut self) {
        if let Some(state) = &self.state {
            let selected = self.list_state.selected().unwrap_or(0);
            match self.view {
                View::Chats | View::Tools | View::Timeline => {
                    let filtered = state.filtered_sessions(&self.search_query);
                    if let Some(sess) = filtered.get(selected)
                        && let Ok(json) = serde_json::to_string_pretty(sess) {
                            let filename = format!("geminiscope_session_{}.json", &sess.session_id[..8]);
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::OpenOptionsExt;
                                use std::io::Write;
                                let mut options = std::fs::OpenOptions::new();
                                options.write(true).create(true).truncate(true).mode(0o600);
                                if let Ok(mut file) = options.open(&filename)
                                    && file.write_all(json.as_bytes()).is_ok() {
                                        self.last_action_msg = Some((format!("󰄬 Exported (Private) to {filename}"), std::time::Instant::now()));
                                    }
                            }
                            #[cfg(not(unix))]
                            {
                                if std::fs::write(&filename, json).is_ok() {
                                    self.last_action_msg = Some((format!("󰄬 Exported to {filename}"), std::time::Instant::now()));
                                }
                            }
                        }
                },
                _ => {}
            }
        }
    }
}
