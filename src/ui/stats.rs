use crate::app::{App, ProjectSort};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, ListItem, Sparkline},
    Frame,
};

use crate::models::State;

pub fn get_stats_list_items<'a>(state: &'a State, sort_mode: ProjectSort) -> Vec<ListItem<'a>> {
    let mut projects = state.projects.clone();

    // Sorting logic
    match sort_mode {
        ProjectSort::Date => {
            // Already sorted by date in parser usually, but let's be explicit if needed
            // Default order is fine.
        }
        ProjectSort::Cost => {
            projects.sort_by(|a, b| {
                let a_cost = state.stats.projects.get(&a.name).map(|s| s.cost).unwrap_or(0.0);
                let b_cost = state.stats.projects.get(&b.name).map(|s| s.cost).unwrap_or(0.0);
                b_cost.partial_cmp(&a_cost).unwrap()
            });
        }
        ProjectSort::Tokens => {
            projects.sort_by(|a, b| {
                let a_tokens = state.stats.projects.get(&a.name).map(|s| s.total_tokens).unwrap_or(0);
                let b_tokens = state.stats.projects.get(&b.name).map(|s| s.total_tokens).unwrap_or(0);
                b_tokens.cmp(&a_tokens)
            });
        }
        ProjectSort::Name => {
            projects.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }

    let mut list = vec![ListItem::new(Line::from("󰓗 All Projects Summary").bold())];
    for p in projects {
        let stats = state.stats.projects.get(&p.name);
        let secondary = if let Some(s) = stats {
            format!("Sessions: {} • ${:.3}", p.sessions.len(), s.cost)
        } else {
            format!("Sessions: {}", p.sessions.len())
        };
        list.push(ListItem::new(vec![
            Line::from(p.name.clone()),
            Line::from(secondary).dark_gray(),
        ]));
    }
    list
}

pub fn render_stats_detail(f: &mut Frame, app: &App, area: Rect) {
    let state = match &app.state {
        Some(s) => s,
        None => return,
    };
    let selected = app.list_state.selected().unwrap_or(0);

    // We need to re-sort here too to match the sidebar if we want to show the correct detail
    let mut projects = state.projects.clone();
    match app.sort_mode {
        ProjectSort::Date => {}
        ProjectSort::Cost => {
            projects.sort_by(|a, b| {
                let a_cost = state.stats.projects.get(&a.name).map(|s| s.cost).unwrap_or(0.0);
                let b_cost = state.stats.projects.get(&b.name).map(|s| s.cost).unwrap_or(0.0);
                b_cost.partial_cmp(&a_cost).unwrap()
            });
        }
        ProjectSort::Tokens => {
            projects.sort_by(|a, b| {
                let a_tokens = state.stats.projects.get(&a.name).map(|s| s.total_tokens).unwrap_or(0);
                let b_tokens = state.stats.projects.get(&b.name).map(|s| s.total_tokens).unwrap_or(0);
                b_tokens.cmp(&a_tokens)
            });
        }
        ProjectSort::Name => {
            projects.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }

    let stats_data = if selected == 0 {
        &state.stats.overall
    } else {
        projects.get(selected - 1).and_then(|p| state.stats.projects.get(&p.name)).unwrap_or(&state.stats.overall)
    };

    let title = if selected == 0 { " Global Summary ".to_string() } else { format!(" {} ", stats_data.name) };

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

    let mut markdown = format!("# Workspace: {}\n\n- **Cost**: ${:.4}\n- **Tokens**: {}\n- **Input**: {}\n- **Output**: {}\n\n### Model Usage\n", 
        stats_data.name, stats_data.cost, stats_data.total_tokens, stats_data.input, stats_data.output);
    for (m, c) in &stats_data.models { markdown.push_str(&format!("- **{}**: {} turns\n", m, c)); }

    crate::ui::render_markdown(f, app, stats_chunks[1], &title, &markdown);
}
