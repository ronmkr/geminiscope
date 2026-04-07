use crate::app::App;
use crate::models::{ProjectSort, State};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, ListItem, Sparkline},
    Frame,
};

pub fn get_stats_list_items(state: &State, sort_mode: ProjectSort) -> Vec<ListItem<'_>> {
    let mut projects = state.projects.clone();

    // Sorting logic
    match sort_mode {
        ProjectSort::Date => {
            projects.sort_by(|a, b| {
                let a_last = a.sessions.iter().map(|s| s.last_updated).max();
                let b_last = b.sessions.iter().map(|s| s.last_updated).max();
                b_last.cmp(&a_last)
            });
        }
        ProjectSort::Cost => {
            projects.sort_by(|a, b| {
                let a_cost = state.stats.projects.get(&a.name).map_or(0.0, |s| s.cost);
                let b_cost = state.stats.projects.get(&b.name).map_or(0.0, |s| s.cost);
                b_cost.partial_cmp(&a_cost).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        ProjectSort::Tokens => {
            projects.sort_by(|a, b| {
                let a_tokens = state.stats.projects.get(&a.name).map_or(0, |s| s.total_tokens);
                let b_tokens = state.stats.projects.get(&b.name).map_or(0, |s| s.total_tokens);
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
        ProjectSort::Date => {
            projects.sort_by(|a, b| {
                let a_last = a.sessions.iter().map(|s| s.last_updated).max();
                let b_last = b.sessions.iter().map(|s| s.last_updated).max();
                b_last.cmp(&a_last)
            });
        }
        ProjectSort::Cost => {
            projects.sort_by(|a, b| {
                let a_cost = state.stats.projects.get(&a.name).map_or(0.0, |s| s.cost);
                let b_cost = state.stats.projects.get(&b.name).map_or(0.0, |s| s.cost);
                b_cost.partial_cmp(&a_cost).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        ProjectSort::Tokens => {
            projects.sort_by(|a, b| {
                let a_tokens = state.stats.projects.get(&a.name).map_or(0, |s| s.total_tokens);
                let b_tokens = state.stats.projects.get(&b.name).map_or(0, |s| s.total_tokens);
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

    let mut markdown = format!("**Cost**: ${:.4} • **Tokens**: {}\n**Input**: {} • **Output**: {}\n### Model Usage\n", 
        stats_data.cost, stats_data.total_tokens, stats_data.input, stats_data.output);
    for (m, c) in &stats_data.models { markdown.push_str(&format!("- **{m}**: {c} turns\n")); }

    crate::ui::render_markdown(f, app, stats_chunks[1], &title, &markdown);
}
