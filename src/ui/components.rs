use crate::app::{App, ProjectSort};
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
    let p = Paragraph::new(lines).alignment(Alignment::Center)
        .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::Rgb(49, 50, 68))));
    f.render_widget(p, area);
}

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(" 󰌒  1-9 Views • j/k List • J/K Scroll • / Search • s Sort • q Quit ", Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White))
    ];
    
    if app.is_searching {
        spans.push(Span::styled(format!(" 󰍉 /{} ", app.search_query), Style::default().bg(Color::Magenta).fg(Color::White).bold()));
    } else if !app.search_query.is_empty() {
        spans.push(Span::styled(format!(" 󰍉 Filter: {} (Esc to clear) ", app.search_query), Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::Cyan)));
    }

    // Add sort indicator if in Stats view
    if app.view == crate::app::View::Stats {
        let sort_label = match app.sort_mode {
            ProjectSort::Date => "Date",
            ProjectSort::Cost => "Cost",
            ProjectSort::Tokens => "Tokens",
            ProjectSort::Name => "Name",
        };
        spans.push(Span::styled(format!(" 󰒺 Sort: {} ", sort_label), Style::default().bg(Color::Cyan).fg(Color::Black).bold()));
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

pub fn format_md_content(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("\n\n");
    }
    format!("```json\n{}\n```", serde_json::to_string_pretty(content).unwrap_or_default())
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
