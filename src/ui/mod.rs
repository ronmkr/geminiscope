pub mod components;
pub mod explorer;
pub mod stats;
pub mod infrastructure;

use crate::app::{App, View};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Span, Text},
    widgets::{Block, Borders, List, Paragraph, Padding},
    Frame,
};
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
        components::render_security_banner(f, app, chunks[0]);
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

/// Shared utility to render markdown content with Termimad
pub fn render_markdown(f: &mut Frame, app: &App, area: Rect, title: &str, markdown: &str) {
    let mut skin = termimad::MadSkin::default();
    skin.bold.set_fg(crossterm::style::Color::Magenta);
    skin.italic.set_fg(crossterm::style::Color::Cyan);
    skin.set_headers_fg(crossterm::style::Color::Magenta);
    let termimad_string = skin.term_text(markdown).to_string();
    
    let content_text = termimad_string.into_text().unwrap_or_else(|_| Text::from(markdown));
    
    let p = Paragraph::new(content_text)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .block(Block::default()
            .title(Span::styled(format!(" {} ", title), Style::default().fg(Color::Magenta).bold()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(49, 50, 68)))
            .padding(Padding::uniform(1)));
    f.render_widget(p, area);
}
