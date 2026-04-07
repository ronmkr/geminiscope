use crate::app::{App, View};
use ratatui::{
    layout::Rect,
    style::{Color, Stylize},
    text::Line,
    widgets::ListItem,
    Frame,
};
use std::fs;

use crate::models::State;

pub fn get_infra_list_items<'a>(state: &'a State, view: View) -> Vec<ListItem<'a>> {
    match view {
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
            let mut groups: std::collections::HashMap<String, Vec<&crate::models::HealthIssue>> = std::collections::HashMap::new();
            for h in &state.health { groups.entry(h.rule.clone()).or_default().push(h); }
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
        View::Skills => state.skills.iter().map(|s| {
            ListItem::new(vec![Line::from(s.name.as_str()).bold(), Line::from(s.extension.as_str()).dark_gray()])
        }).collect(),
        View::MCP => state.mcp_servers.iter().map(|s| {
            ListItem::new(vec![Line::from(s.name.as_str()).bold(), Line::from(if s.url.is_some() { "Remote" } else { "Local" }).dark_gray()])
        }).collect(),
        View::Settings => {
            if let Some(obj) = state.settings.as_object() {
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();
                keys.into_iter().map(|k| ListItem::new(vec![Line::from(k.as_str()).bold(), Line::from("Configuration section").dark_gray()])).collect()
            } else {
                vec![ListItem::new(Line::from("No settings found").dark_gray())]
            }
        },
        _ => Vec::new(),
    }
}

pub fn render_infra_detail(f: &mut Frame, app: &App, area: Rect) {
    let state = match &app.state {
        Some(s) => s,
        None => return,
    };
    let selected = app.list_state.selected().unwrap_or(0);
    let mut markdown = String::new();
    let mut title = " Detail ".to_string();

    match app.view {
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
        _ => {}
    }

    crate::ui::render_markdown(f, app, area, &title, &markdown);
}
