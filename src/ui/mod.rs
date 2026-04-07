pub mod components;
pub mod explorer;
pub mod stats;
pub mod infrastructure;

use crate::app::{App, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Span, Text, Line},
    widgets::{Block, Borders, List, Paragraph, Padding},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let has_critical = if let Some(state) = &app.state {
        state.health.iter().any(|h| h.severity == "Critical")
    } else {
        false
    };

    let has_notif = app.last_action_msg.as_ref().map(|(_, t)| t.elapsed().as_secs() < 3).unwrap_or(false);

    let mut constraints = vec![Constraint::Min(0), Constraint::Length(1)];
    if has_critical { constraints.insert(0, Constraint::Length(1)); }
    if has_notif { constraints.insert(0, Constraint::Length(1)); }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    let mut current_idx = 0;
    if has_notif {
        if let Some((msg, _)) = &app.last_action_msg {
            let p = Paragraph::new(Line::from(Span::styled(format!(" {} ", msg), Style::default().bg(Color::Green).fg(Color::Black).bold())));
            f.render_widget(p, chunks[current_idx]);
            current_idx += 1;
        }
    }

    if has_critical {
        components::render_security_banner(f, app, chunks[current_idx]);
        current_idx += 1;
    }

    let main_area = chunks[current_idx];
    let footer_area = chunks[current_idx + 1];

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(6),  // Rail
            Constraint::Length(32), // Sidebar
            Constraint::Min(0),     // Content
        ])
        .split(main_area);

    components::render_rail(f, app, main_layout[0]);
    render_sidebar(f, app, main_layout[1]);
    render_content(f, app, main_layout[2]);
    components::render_footer(f, app, footer_area);
}

fn render_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    if app.is_loading {
        f.render_widget(Paragraph::new("\n\n󰄦 Loading...").dark_gray().alignment(ratatui::layout::Alignment::Center), area);
        return;
    }
    
    let view = app.view;
    let search_query = &app.search_query;
    let state = match &app.state {
        Some(s) => s,
        None => return,
    };
    let sort_mode = app.sort_mode;

    let title = match view {
        View::Chats => " CHATS ",
        View::Stats => " PROJECTS ",
        View::Tools => " TOOLS ",
        View::Memory => " MEMORY ",
        View::Plans => " PLANS ",
        View::Health => " HEALTH ",
        View::Timeline => " TIMELINE ",
        View::Skills => " SKILLS ",
        View::MCP => " MCPS ",
        View::Settings => " SETTINGS ",
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(Color::Magenta).bold()))
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::Rgb(49, 50, 68)))
        .padding(Padding::horizontal(1));

    let items = match view {
        View::Chats | View::Tools | View::Timeline => explorer::get_explorer_list_items(state, view, search_query),
        View::Stats => stats::get_stats_list_items(state, sort_mode),
        _ => infrastructure::get_infra_list_items(state, view),
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White))
        .highlight_symbol(" ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    if app.is_loading || app.state.is_none() { return; }
    
    match app.view {
        View::Chats | View::Tools | View::Timeline => explorer::render_explorer_detail(f, app, area),
        View::Stats => stats::render_stats_detail(f, app, area),
        _ => infrastructure::render_infra_detail(f, app, area),
    }
}

/// Shared utility to render high-density text content with basic syntax highlighting
pub fn render_markdown(f: &mut Frame, app: &App, area: Rect, title: &str, markdown: &str) {
    // Increased limit to 500KB but optimized line loop
    if markdown.len() > 500_000 {
        let p = Paragraph::new(format!("[Content too large for TUI rendering: {} bytes. Press 'e' to export full session.]", markdown.len()))
            .block(Block::default().title(format!(" {} ", title)).borders(Borders::ALL));
        f.render_widget(p, area);
        return;
    }

    let mut lines = Vec::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { 
            // Avoid multiple empty lines
            if lines.last().map(|l: &Line| l.spans.is_empty()).unwrap_or(true) {
                continue;
            }
            lines.push(Line::from(""));
            continue;
        }

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from("────────────────────────────────────────────────────────────────").dark_gray());
            continue;
        }

        if trimmed.starts_with("# ") {
            lines.push(Line::from(Span::styled(trimmed[2..].to_string(), Style::default().fg(Color::Magenta).bold())));
        } else if trimmed.starts_with("## ") {
            lines.push(Line::from(Span::styled(trimmed[3..].to_string(), Style::default().fg(Color::Magenta).bold())));
        } else if trimmed.starts_with("### ") {
            lines.push(Line::from(Span::styled(trimmed[4..].to_string(), Style::default().fg(Color::Cyan).bold())));
        } else if trimmed == "---" {
            lines.push(Line::from("────────────────────────────────────────────────────────────────").dark_gray());
        } else if in_code_block || (trimmed.starts_with('{') || trimmed.starts_with('"') || trimmed.starts_with('}')) {
            // Basic JSON/Code highlighting
            let mut spans = Vec::new();
            if trimmed.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                spans.push(Span::styled(parts[0].to_string(), Style::default().fg(Color::Cyan)));
                spans.push(Span::raw(":"));
                if parts.len() > 1 {
                    spans.push(Span::styled(parts[1].to_string(), Style::default().fg(Color::Yellow)));
                }
            } else {
                spans.push(Span::styled(line.to_string(), Style::default().fg(Color::Rgb(200, 200, 200))));
            }
            lines.push(Line::from(spans));
        } else {
            // Regular text with bold support
            if trimmed.contains("**") {
                let mut spans = Vec::new();
                let parts: Vec<&str> = line.split("**").collect();
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 1 {
                        spans.push(Span::styled(part.to_string(), Style::default().bold().fg(Color::Yellow)));
                    } else {
                        spans.push(Span::raw(part.to_string()));
                    }
                }
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::from(line.to_string()));
            }
        }
    }

    let p = Paragraph::new(Text::from(lines))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .block(Block::default()
            .title(Span::styled(format!(" {} ", title), Style::default().fg(Color::Magenta).bold()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(49, 50, 68)))
            .padding(Padding::uniform(1)));
    f.render_widget(p, area);
}
