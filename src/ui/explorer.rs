use crate::app::{App, View};
use crate::ui::components::{format_session_search, format_raw_content, format_md_content};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::ListItem,
    Frame,
};

use crate::models::State;

pub fn get_explorer_list_items<'a>(state: &'a State, view: View, search_query: &str) -> Vec<ListItem<'a>> {
    match view {
        View::Chats | View::Tools => {
            let filtered: Vec<_> = if search_query.is_empty() {
                state.all_sessions.iter().collect()
            } else {
                let query = search_query.to_lowercase();
                state.all_sessions.iter().filter(|s| {
                    format_session_search(s).to_lowercase().contains(&query)
                }).collect()
            };

            filtered.into_iter().map(|s| {
                let first_msg = s.messages.iter().find(|m| m.msg_type == "user")
                    .map(|m| format_raw_content(&m.content))
                    .unwrap_or_else(|| "Empty Session".to_string());
                let title = if first_msg.len() > 25 { format!("{}...", &first_msg[..25]) } else { first_msg };
                ListItem::new(vec![
                    Line::from(title).bold(),
                    Line::from(s.last_updated.format("%b %d, %H:%M").to_string()).dark_gray(),
                ])
            }).collect()
        },
        View::Timeline => state.timeline.iter().map(|e| {
            let preview = e.session.messages.iter().find(|m| m.msg_type == "user")
                .map(|m| format_raw_content(&m.content))
                .unwrap_or_else(|| "Empty".to_string());
            let title = if preview.len() > 20 { format!("{}...", &preview[..20]) } else { preview };
            ListItem::new(vec![
                Line::from(title).bold(),
                Line::from(e.project.as_str()).dark_gray(),
            ])
        }).collect(),
        _ => Vec::new(),
    }
}

pub fn render_explorer_detail(f: &mut Frame, app: &App, area: Rect) {
    let state = match &app.state {
        Some(s) => s,
        None => return,
    };
    let selected = app.list_state.selected().unwrap_or(0);
    let mut markdown = String::new();
    let mut title = " Detail ".to_string();

    match app.view {
        View::Chats => {
            let filtered: Vec<_> = if app.search_query.is_empty() {
                state.all_sessions.iter().collect()
            } else {
                let query = app.search_query.to_lowercase();
                state.all_sessions.iter().filter(|s| {
                    format_session_search(s).to_lowercase().contains(&query)
                }).collect()
            };

            if let Some(sess) = filtered.get(selected) {
                title = format!(" Chat: {} ", &sess.session_id[..8]);
                for msg in &sess.messages {
                    let header = if msg.msg_type == "user" { "### 󰭻 USER" } else { "### 󰄦 GEMINI" };
                    markdown.push_str(&format!("{}\n", header));
                    if let (Some(model), Some(tokens)) = (&msg.model, &msg.tokens) {
                        let pricing = if model.contains("flash") { (0.075, 0.30) } else { (3.50, 10.50) };
                        let cost = (tokens.input as f64 / 1_000_000.0 * pricing.0) + (tokens.output as f64 / 1_000_000.0 * pricing.1);
                        markdown.push_str(&format!("*Model: {} • Tokens: {} • Cost: ${:.4}*\n\n", model, tokens.total, cost));
                    }
                    markdown.push_str(&format_md_content(&msg.content));
                    markdown.push_str("\n\n---\n\n");
                }
            }
        },
        View::Tools => {
            let filtered: Vec<_> = if app.search_query.is_empty() {
                state.all_sessions.iter().collect()
            } else {
                let query = app.search_query.to_lowercase();
                state.all_sessions.iter().filter(|s| {
                    format_session_search(s).to_lowercase().contains(&query)
                }).collect()
            };

            if let Some(sess) = filtered.get(selected) {
                title = " Tool History ".to_string();
                for msg in &sess.messages {
                    if let Some(calls) = &msg.tool_calls {
                        for tc in calls {
                            markdown.push_str(&format!("### {} `{}`\n", tc.display_name.as_deref().unwrap_or(&tc.name), tc.status));
                            markdown.push_str(&format!("**Arguments**:\n```json\n{}\n```\n\n", serde_json::to_string_pretty(&tc.args).unwrap_or_default()));
                            if let Some(res) = &tc.result {
                                markdown.push_str(&format!("**Result**:\n```json\n{}\n```\n\n", serde_json::to_string_pretty(res).unwrap_or_default()));
                            }
                        }
                    }
                }
            }
        },
        View::Timeline => {
            if let Some(e) = state.timeline.get(selected) {
                title = format!(" Timeline: {} ", e.project);
                markdown = format!("# Event in {}\n\n- **Session ID**: {}\n- **Time**: {}\n\n---\n\n", 
                    e.project, e.session.session_id, e.session.last_updated);
                for msg in &e.session.messages {
                    markdown.push_str(&format!("### {}\n{}\n\n", msg.msg_type, format_raw_content(&msg.content)));
                }
            }
        },
        _ => {}
    }

    crate::ui::render_markdown(f, app, area, &title, &markdown);
}
