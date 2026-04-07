use crate::app::{App, ProjectSort};
use crate::ui::theme;
use ratatui::{
    layout::{Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_security_banner(f: &mut Frame, app: &App, area: Rect) {
    if let Some(state) = &app.state {
        let critical_count = state.health.iter().filter(|h| h.severity == "Critical").count();
        let text = format!(" 󰒐 CRITICAL SECURITY ALERT: {} issues detected in history. Check health view (6) immediately! ", critical_count);
        let p = Paragraph::new(Line::from(Span::styled(text, Style::default().bg(Color::Red).fg(Color::White).bold())));
        f.render_widget(p, area);
    }
}

pub fn render_rail(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.state.as_ref().map(|s| s.theme.clone()).unwrap_or_default();
    let primary_color = theme::get_color(&theme.primary);
    let sidebar_bg = theme::get_color(&theme.sidebar_bg);

    let icons = vec!["󰭻", "󰄦", "󰓙", "󰤄", "󰏚", "󰓚", "󰃭", "󰄦", "󰒄", "󰒓"];
    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for (i, icon) in icons.iter().enumerate() {
        let style = if (app.view as usize) == i {
            Style::default().fg(primary_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled(*icon, style)));
        lines.push(Line::from(""));
    }
    let p = Paragraph::new(lines).alignment(Alignment::Center)
        .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(sidebar_bg)));
    f.render_widget(p, area);
}

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.state.as_ref().map(|s| s.theme.clone()).unwrap_or_default();
    let sidebar_bg = theme::get_color(&theme.sidebar_bg);
    let primary_color = theme::get_color(&theme.primary);
    let secondary_color = theme::get_color(&theme.secondary);

    let mut spans = vec![
        Span::styled(" 󰌒  1-9 View • j/k List • J/K Scroll • / Search • s Sort • e Export • q Quit • (Hold Shift to select text) ", Style::default().bg(sidebar_bg).fg(Color::White))
    ];
    
    if app.is_searching {
        spans.push(Span::styled(format!(" 󰍉 /{} ", app.search_query), Style::default().bg(primary_color).fg(Color::White).bold()));
    } else if !app.search_query.is_empty() {
        spans.push(Span::styled(format!(" 󰍉 Filter: {} (Esc to clear) ", app.search_query), Style::default().bg(sidebar_bg).fg(secondary_color)));
    }

    // Add sort indicator if in Stats view
    if app.view == crate::app::View::Stats {
        let sort_label = match app.sort_mode {
            ProjectSort::Date => "Date",
            ProjectSort::Cost => "Cost",
            ProjectSort::Tokens => "Tokens",
            ProjectSort::Name => "Name",
        };
        spans.push(Span::styled(format!(" 󰒺 Sort: {} ", sort_label), Style::default().bg(secondary_color).fg(Color::Black).bold()));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn format_raw_content(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    "".to_string()
}

fn truncate_json_strings(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::String(s) => {
            // Trim whitespace first to avoid empty space gaps
            let trimmed = s.trim();
            let newlines = trimmed.matches('\n').count();
            if trimmed.len() > 1000 || newlines > 20 {
                let truncated = trimmed.chars().take(1000).collect::<String>();
                *s = format!("{} ...\n[Truncated {} bytes, {} lines. Press 'e' to export full session.]", truncated, s.len(), newlines);
            } else {
                *s = trimmed.to_string();
            }
        },

        serde_json::Value::Array(a) => {
            for item in a {
                truncate_json_strings(item);
            }
        },
        serde_json::Value::Object(o) => {
            for (_, val) in o.iter_mut() {
                truncate_json_strings(val);
            }
        },
        _ => {}
    }
}

pub fn clean_json(v: &serde_json::Value) -> String {
    let mut cloned = v.clone();
    truncate_json_strings(&mut cloned);
    let pretty = serde_json::to_string_pretty(&cloned).unwrap_or_default();
    pretty.lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_md_content(content: &serde_json::Value) -> String {
    let raw = if let Some(s) = content.as_str() { s.to_string() }
    else if let Some(arr) = content.as_array() {
        arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("\n")
    } else {
        let js = content.to_string();
        if js.len() > 10_000 {
            format!("[Large Content: {} bytes]", js.len())
        } else {
            format!("```json\n{}\n```", serde_json::to_string_pretty(content).unwrap_or_default())
        }
    };
    raw.trim().to_string()
}

pub fn format_session_search(sess: &crate::models::Session) -> String {
    let mut text = String::new();
    for msg in &sess.messages {
        text.push_str(&format_value(&msg.content));
    }
    text
}

pub fn format_value(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    format!("{}", content)
}
