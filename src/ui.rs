use crate::app::{App, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Padding, Sparkline},
    Frame,
};
use std::fs;
use ansi_to_tui::IntoText;

pub fn render(f: &mut Frame, app: &mut App) {
    let has_critical = if let Some(state) = &app.state {
        state.health.iter().any(|h| h.severity == "Critical")
    } else {
        false
    };

    let top_constraints = if has_critical {
        vec![Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)]
    } else {
        vec![Constraint::Min(0), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(top_constraints)
        .split(f.area());

    let (main_area, footer_area) = if has_critical {
        render_security_banner(f, app, chunks[0]);
        (chunks[1], chunks[2])
    } else {
        (chunks[0], chunks[1])
    };

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(6),  // Rail
            Constraint::Length(32), // Sidebar
            Constraint::Min(0),     // Content
        ])
        .split(main_area);

    render_rail(f, app, main_layout[0]);
    render_sidebar(f, app, main_layout[1]);
    render_content(f, app, main_layout[2]);
    render_footer(f, app, footer_area);
}

fn render_security_banner(f: &mut Frame, app: &App, area: Rect) {
    if let Some(state) = &app.state {
        let critical_count = state.health.iter().filter(|h| h.severity == "Critical").count();
        let text = format!(" 󰒐 CRITICAL SECURITY ALERT: {} issues detected in history. Check health view (6) immediately! ", critical_count);
        let p = Paragraph::new(Line::from(Span::styled(text, Style::default().bg(Color::Red).fg(Color::White).bold())));
        f.render_widget(p, area);
    }
}

fn render_rail(f: &mut Frame, app: &App, area: Rect) {
    let icons = vec!["󰭻", "󰄦", "󰓙", "󰤄", "󰏚", "󰓚", "󰃭", "󰄦", "󰒄", "󰒓"];
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for (i, icon) in icons.iter().enumerate() {
        let style = if (app.view as usize) == i {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled(*icon, style)));
        lines.push(Line::from(""));
    }
    let p = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::Rgb(49, 50, 68))));
    f.render_widget(p, area);
}

fn render_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    if app.is_loading {
        f.render_widget(Paragraph::new("\n\n󰄦 Loading...").dark_gray().alignment(ratatui::layout::Alignment::Center), area);
        return;
    }
    let state = match &app.state {
        Some(s) => s,
        None => return,
    };
    let title = match app.view {
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

    let items: Vec<ListItem> = match app.view {
        View::Chats | View::Tools => {
            let filtered: Vec<_> = if app.search_query.is_empty() {
                state.all_sessions.iter().collect()
            } else {
                let query = app.search_query.to_lowercase();
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
        View::Stats => {
            let mut list = vec![ListItem::new(Line::from("󰓗 All Projects Summary").bold())];
            for p in &state.projects { 
                list.push(ListItem::new(vec![
                    Line::from(p.name.as_str()),
                    Line::from(format!("Sessions: {}", p.sessions.len())).dark_gray(),
                ])); 
            }
            list
        },
        View::Memory => {
            let mut items = Vec::new();
            for p in &state.projects {
                for f in &p.memory_files {
                    items.push(ListItem::new(vec![Line::from(f.name.as_str()).bold(), Line::from(p.name.as_str()).dark_gray()]));
                }
            }
            items
        },
        View::Plans => {
            let mut items = Vec::new();
            for p in &state.projects {
                for f in &p.plan_files {
                    items.push(ListItem::new(vec![Line::from(f.name.as_str()).bold(), Line::from(p.name.as_str()).dark_gray()]));
                }
            }
            items
        },
        View::Health => {
            // Group by Rule
            let mut groups: std::collections::HashMap<String, Vec<&crate::models::HealthIssue>> = std::collections::HashMap::new();
            for h in &state.health {
                groups.entry(h.rule.clone()).or_default().push(h);
            }
            let mut sorted_rules: Vec<_> = groups.keys().collect();
            sorted_rules.sort();
            sorted_rules.into_iter().map(|r| {
                let issues = &groups[r];
                let severity = if issues.iter().any(|i| i.severity == "Critical") { Color::Red } 
                            else if issues.iter().any(|i| i.severity == "Warning") { Color::Yellow }
                            else { Color::Cyan };
                ListItem::new(vec![
                    Line::from(r.clone()).bold().fg(severity),
                    Line::from(format!("{} issues", issues.len())).dark_gray(),
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
        View::Skills => state.skills.iter().map(|s| {
            ListItem::new(vec![
                Line::from(s.name.as_str()).bold(),
                Line::from(s.extension.as_str()).dark_gray(),
            ])
        }).collect(),
        View::MCP => state.mcp_servers.iter().map(|s| {
            ListItem::new(vec![
                Line::from(s.name.as_str()).bold(),
                Line::from(if s.url.is_some() { "Remote" } else { "Local" }).dark_gray(),
            ])
        }).collect(),
        View::Settings => {
            if let Some(obj) = state.settings.as_object() {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                keys.into_iter().map(|k| {
                    ListItem::new(vec![
                        Line::from(k.as_str()).bold(),
                        Line::from("Configuration section").dark_gray(),
                    ])
                }).collect()
            } else {
                vec![ListItem::new(Line::from("No settings found").dark_gray())]
            }
        },
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White))
        .highlight_symbol(" ");
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    if app.is_loading || app.state.is_none() { return; }
    let state = app.state.as_ref().unwrap();
    let selected = app.list_state.selected().unwrap_or(0);
    let mut title = " Detail ".to_string();
    let mut markdown = String::new();
    let mut markdown_area = area;

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
        View::Stats => {
            let stats_data = if selected == 0 {
                &state.stats.overall
            } else {
                state.projects.get(selected - 1).and_then(|p| state.stats.projects.get(&p.name)).unwrap_or(&state.stats.overall)
            };

            title = if selected == 0 { " Global Summary ".to_string() } else { format!(" {} ", stats_data.name) };

            let stats_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(0)])
                .split(area);

            // Render Sparkline
            let data: Vec<u64> = stats_data.token_history.iter().map(|h| h.1 as u64).collect();
            let sparkline = Sparkline::default()
                .block(Block::default().title(" Token Usage Trend ").borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Rgb(49, 50, 68))))
                .data(&data)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(sparkline, stats_chunks[0]);

            markdown = format!("# Workspace: {}\n\n- **Cost**: ${:.4}\n- **Tokens**: {}\n- **Input**: {}\n- **Output**: {}\n\n### Model Usage\n", 
                stats_data.name, stats_data.cost, stats_data.total_tokens, stats_data.input, stats_data.output);
            for (m, c) in &stats_data.models { markdown.push_str(&format!("- **{}**: {} turns\n", m, c)); }
            
            markdown_area = stats_chunks[1];
        },
        View::Memory => {
            let mut all_files = Vec::new();
            for p in &state.projects { for f in &p.memory_files { all_files.push(f); } }
            if let Some(f) = all_files.get(selected) {
                title = format!(" {} ", f.name);
                markdown = fs::read_to_string(&f.path).unwrap_or_else(|_| "Error reading file".to_string());
            }
        },
        View::Plans => {
            let mut all_files = Vec::new();
            for p in &state.projects { for f in &p.plan_files { all_files.push(f); } }
            if let Some(f) = all_files.get(selected) {
                title = format!(" {} ", f.name);
                markdown = fs::read_to_string(&f.path).unwrap_or_else(|_| "Error reading file".to_string());
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
        View::Health => {
            let mut groups: std::collections::HashMap<String, Vec<&crate::models::HealthIssue>> = std::collections::HashMap::new();
            for h in &state.health { groups.entry(h.rule.clone()).or_default().push(h); }
            let mut sorted_rules: Vec<_> = groups.keys().collect();
            sorted_rules.sort();
            let critical = state.health.iter().filter(|i| i.severity == "Critical").count();
            let warnings = state.health.iter().filter(|i| i.severity == "Warning").count();
            let score = if critical > 0 { "POOR" } else if warnings > 5 { "FAIR" } else if warnings > 0 { "GOOD" } else { "EXCELLENT" };
            markdown = format!("# Health Score: {}\n\n- **Critical Issues**: {}\n- **Warnings**: {}\n\n---\n\n", score, critical, warnings);
            if let Some(rule) = sorted_rules.get(selected) {
                title = format!(" {} ", rule);
                markdown.push_str(&format!("## Rule: {}\n\n", rule));
                for issue in &groups[*rule] {
                    markdown.push_str(&format!("### {}\n- **Project**: {}\n- **File**: {}\n- **Severity**: {}\n\n", 
                        issue.message, issue.project, issue.file.as_deref().unwrap_or("N/A"), issue.severity));
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
        View::Skills => {
            if let Some(s) = state.skills.get(selected) {
                title = format!(" Skill: {} ", s.name);
                markdown = format!("# Skill: {}\n**Extension**: {}\n\n> {}\n\n---\n\n## Definition\n```toml\n{}\n```", 
                    s.name, s.extension, s.description, s.content);
            }
        },
        View::MCP => {
            if let Some(s) = state.mcp_servers.get(selected) {
                title = format!(" MCP: {} ", s.name);
                markdown = format!("# MCP Server: {}\n\n", s.name);
                if let Some(url) = &s.url { markdown.push_str(&format!("- **Remote URL**: {}\n", url)); }
                if let Some(cmd) = &s.command { markdown.push_str(&format!("- **Local Command**: `{}`\n", cmd)); }
                if !s.args.is_empty() { markdown.push_str(&format!("- **Arguments**: `{}`\n", s.args.join(" "))); }
                if !s.env.is_empty() {
                    markdown.push_str("\n### Environment Variables\n");
                    for (k, v) in &s.env { markdown.push_str(&format!("- **{}**: `{}`\n", k, v)); }
                }
            }
        },
        View::Settings => {
            if let Some(obj) = state.settings.as_object() {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                if let Some(key) = keys.get(selected) {
                    title = format!(" Settings: {} ", key);
                    markdown = format!("# {}\n\n", key);
                    if let Some(val) = obj.get(*key) {
                        markdown.push_str(&format!("```json\n{}\n```\n", serde_json::to_string_pretty(val).unwrap_or_default()));
                    }
                }
            } else {
                markdown = "No settings data available.".to_string();
            }
        },
    }

    // 1. Render Markdown to ANSI using Termimad
    let mut skin = termimad::MadSkin::default();
    skin.bold.set_fg(crossterm::style::Color::Magenta);
    skin.italic.set_fg(crossterm::style::Color::Cyan);
    skin.set_headers_fg(crossterm::style::Color::Magenta);
    let termimad_string = skin.term_text(&markdown).to_string();
    
    // 2. Convert ANSI string to Ratatui Text using ansi-to-tui
    let content_text = termimad_string.into_text().unwrap_or_else(|_| Text::from(markdown));
    
    let p = Paragraph::new(content_text)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .block(Block::default()
            .title(Span::styled(format!(" {} ", title), Style::default().fg(Color::Magenta).bold()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(49, 50, 68)))
            .padding(Padding::uniform(1)));
    f.render_widget(p, markdown_area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(" 󰌒  1-9 Views • j/k List • J/K Scroll • / Search • q Quit ", Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White))
    ];
    
    if app.is_searching {
        spans.push(Span::styled(format!(" 󰍉 /{} ", app.search_query), Style::default().bg(Color::Magenta).fg(Color::White).bold()));
    } else if !app.search_query.is_empty() {
        spans.push(Span::styled(format!(" 󰍉 Filter: {} (Esc to clear) ", app.search_query), Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::Cyan)));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
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

fn format_raw_content(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    "".to_string()
}

fn format_md_content(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("\n\n");
    }
    format!("```json\n{}\n```", serde_json::to_string_pretty(content).unwrap_or_default())
}
