use crate::app::App;
use crate::models::ProjectSort;
use crate::ui::theme;
use ratatui::{
    layout::{Rect, Alignment},
    style::{Color, Modifier, Style, Stylize},
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

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for view in crate::models::View::all() {
        let style = if app.view == view {
            Style::default().fg(primary_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(Span::styled(view.icon().to_string(), style)));
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

    let mut spans = if app.is_editing_setting {
        vec![
            Span::styled(" 󰏫 EDIT MODE: Type value and press Enter to save, Esc to cancel ", Style::default().bg(Color::Yellow).fg(Color::Black).bold()),
            Span::styled(format!("  {} > ", app.setting_path.last().unwrap_or(&"setting".to_string())), Style::default().bg(sidebar_bg).fg(Color::White)),
            Span::styled(format!(" {} ", app.edit_input), Style::default().bg(sidebar_bg).fg(primary_color).bold()),
            Span::styled("█", Style::default().fg(primary_color).add_modifier(Modifier::SLOW_BLINK)),
        ]
    } else {
        vec![
            Span::styled(" 󰌒  1-9 View • j/k List • J/K Scroll • / Search • d Diff • o Open • s Sort • e Export • ?: Help • q Quit ", Style::default().bg(sidebar_bg).fg(Color::White))
        ]
    };
    
    if !app.is_editing_setting {
        if app.is_searching {
            spans.push(Span::styled(format!(" 󰍉 /{} ", app.search_query), Style::default().bg(primary_color).fg(Color::White).bold()));
        } else if !app.search_query.is_empty() {
            spans.push(Span::styled(format!(" 󰍉 Filter: {} (Esc to clear) ", app.search_query), Style::default().bg(sidebar_bg).fg(secondary_color)));
        }

        // Add sort indicator if in Stats view
        if app.view == crate::models::View::Stats {
            let sort_label = match app.sort_mode {
                ProjectSort::Date => "Date",
                ProjectSort::Cost => "Cost",
                ProjectSort::Tokens => "Tokens",
                ProjectSort::Name => "Name",
            };
            spans.push(Span::styled(format!(" 󰒺 Sort: {} ", sort_label), Style::default().bg(secondary_color).fg(Color::Black).bold()));
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn render_help_modal(f: &mut Frame, app: &App) {
    let area = f.area();
    let vertical = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(60),
            ratatui::layout::Constraint::Percentage(20),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(25),
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(25),
        ])
        .split(vertical[1]);
    let help_area = horizontal[1];

    let theme = app.state.as_ref().map(|s| s.theme.clone()).unwrap_or_default();
    let primary_color = theme::get_color(&theme.primary);

    let block = Block::default()
        .title(Span::styled(" 󰋗 GEMINISCOPE HELP ", Style::default().fg(primary_color).bold()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(primary_color))
        .bg(Color::Rgb(20, 20, 20));

    let help_text = vec![
        Line::from(vec![Span::styled(" Navigation ", Style::default().fg(primary_color).bold())]),
        Line::from(vec![Span::styled("  j / ↓   ", Style::default().fg(Color::Yellow)), Span::raw("  Move cursor down")]),
        Line::from(vec![Span::styled("  k / ↑   ", Style::default().fg(Color::Yellow)), Span::raw("  Move cursor up")]),
        Line::from(vec![Span::styled("  J / K   ", Style::default().fg(Color::Yellow)), Span::raw("  Scroll detail view")]),
        Line::from(vec![Span::styled("  Alt+j/k ", Style::default().fg(Color::Yellow)), Span::raw("  Precise scroll")]),
        Line::from(""),
        Line::from(vec![Span::styled(" Views ", Style::default().fg(primary_color).bold())]),
        Line::from(vec![Span::styled("  1       ", Style::default().fg(Color::Yellow)), Span::raw("  Chats (Conversations)")]),
        Line::from(vec![Span::styled("  2       ", Style::default().fg(Color::Yellow)), Span::raw("  Stats (Costs/Tokens)")]),
        Line::from(vec![Span::styled("  3       ", Style::default().fg(Color::Yellow)), Span::raw("  Tools (MDC/Functions)")]),
        Line::from(vec![Span::styled("  4-9     ", Style::default().fg(Color::Yellow)), Span::raw("  Memory, Plans, Health, etc")]),
        Line::from(vec![Span::styled("  0       ", Style::default().fg(Color::Yellow)), Span::raw("  Settings")]),
        Line::from(""),
        Line::from(vec![Span::styled(" Actions ", Style::default().fg(primary_color).bold())]),
        Line::from(vec![Span::styled("  /       ", Style::default().fg(Color::Yellow)), Span::raw("  Search/Filter current view")]),
        Line::from(vec![Span::styled("  o       ", Style::default().fg(Color::Yellow)), Span::raw("  Open selected file in editor")]),
        Line::from(vec![Span::styled("  d       ", Style::default().fg(Color::Yellow)), Span::raw("  Diff: Press on 1st then 2nd session")]),
        Line::from(vec![Span::styled("  s       ", Style::default().fg(Color::Yellow)), Span::raw("  Toggle sort (Stats view)")]),
        Line::from(vec![Span::styled("  Ctrl+r  ", Style::default().fg(Color::Yellow)), Span::raw("  Toggle Secret Redaction")]),
        Line::from(vec![Span::styled("  e       ", Style::default().fg(Color::Yellow)), Span::raw("  Export view to JSON file")]),
        Line::from(vec![Span::styled("  Enter   ", Style::default().fg(Color::Yellow)), Span::raw("  Edit selected setting")]),
        Line::from(vec![Span::styled("  q / Esc ", Style::default().fg(Color::Yellow)), Span::raw("  Close Help / Quit App")]),
        Line::from(""),
        Line::from(vec![Span::styled(" Icons ", Style::default().fg(primary_color).bold())]),
        Line::from(vec![Span::raw("  󰭻 Chats  󰄦 Stats  󰓙 Tools  󰤄 Memory  󰏚 Plans ")]),
        Line::from(vec![Span::raw("  󰓚 Health  󰃭 Timeline  󰛨 Skills  󰒄 MCP  󰒓 Settings ")]),
    ];

    let p = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: false });

    f.render_widget(ratatui::widgets::Clear, help_area); // Clear background
    f.render_widget(p, help_area);
}

use ratatui::layout::Layout;

pub fn render_setting_edit_modal(f: &mut Frame, app: &App) {
    let area = f.area();
    let vertical = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(40),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Percentage(40),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(20),
            ratatui::layout::Constraint::Percentage(60),
            ratatui::layout::Constraint::Percentage(20),
        ])
        .split(vertical[1]);
    let input_area = horizontal[1];

    let theme = app.state.as_ref().map(|s| s.theme.clone()).unwrap_or_default();
    let primary_color = theme::get_color(&theme.primary);

    let title = format!(" 󰏫 Edit Setting: {} ", app.setting_path.last().unwrap_or(&"setting".to_string()));
    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(primary_color).bold()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(primary_color))
        .bg(Color::Rgb(30, 30, 30));

    let p = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {} ", app.edit_input), Style::default().fg(Color::White).bold()),
        Span::styled("█", Style::default().fg(primary_color).add_modifier(Modifier::SLOW_BLINK)),
    ])).block(block);

    f.render_widget(ratatui::widgets::Clear, input_area);
    f.render_widget(p, input_area);
}

fn truncate_json_strings(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            let newlines = trimmed.matches('\n').count();
            if trimmed.len() > 1000 || newlines > 20 {
                let truncated = trimmed.chars().take(1000).collect::<String>();
                *s = format!("{} ...\n[Truncated {} bytes, {} lines. Press 'e' to export full session.]", truncated, s.len(), newlines);
            } else {
                *s = trimmed.to_string();
            }
        },
        serde_json::Value::Array(a) => { for item in a { truncate_json_strings(item); } },
        serde_json::Value::Object(o) => { for (_, val) in o.iter_mut() { truncate_json_strings(val); } },
        _ => {}
    }
}

pub fn clean_json(v: &serde_json::Value) -> String {
    let mut cloned = v.clone();
    truncate_json_strings(&mut cloned);
    let pretty = serde_json::to_string_pretty(&cloned).unwrap_or_default();
    pretty.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n")
}

pub fn format_md_content(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { s.trim().to_string() }
    else if let Some(arr) = content.as_array() {
        arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("\n").trim().to_string()
    } else {
        let js = content.to_string();
        if js.len() > 10_000 { format!("[Large Content: {} bytes]", js.len()) }
        else { format!("```json\n{}\n```", serde_json::to_string_pretty(content).unwrap_or_default()) }
    }
}
