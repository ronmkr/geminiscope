use crate::app::{App, View};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::ListItem,
    Frame,
};
use std::fs;
use crate::models::State;

pub struct SettingSchema {
    pub path: &'static str,
    pub description: &'static str,
    pub default_val: serde_json::Value,
}

pub fn get_known_settings() -> Vec<SettingSchema> {
    vec![
        SettingSchema { path: "general.vimMode", description: "Enable Vim keybindings.", default_val: serde_json::Value::Bool(false) },
        SettingSchema { path: "general.preferredEditor", description: "Preferred editor command (e.g. 'code', 'vim').", default_val: serde_json::Value::String("vim".to_string()) },
        SettingSchema { path: "general.enableAutoUpdate", description: "Enable automatic updates.", default_val: serde_json::Value::Bool(true) },
        SettingSchema { path: "general.enableNotifications", description: "Enable OS notifications (macOS).", default_val: serde_json::Value::Bool(false) },
        SettingSchema { path: "ui.theme", description: "UI color theme (e.g. 'GitHub', 'Monokai').", default_val: serde_json::Value::String("GitHub".to_string()) },
        SettingSchema { path: "ui.showLineNumbers", description: "Show line numbers in chat.", default_val: serde_json::Value::Bool(true) },
        SettingSchema { path: "ui.inlineThinkingMode", description: "Show model thinking inline (off, full).", default_val: serde_json::Value::String("off".to_string()) },
        SettingSchema { path: "model.name", description: "Default Gemini model ID.", default_val: serde_json::Value::String("gemini-1.5-pro".to_string()) },
        SettingSchema { path: "context.fileFiltering.respectGitIgnore", description: "Respect .gitignore rules.", default_val: serde_json::Value::Bool(true) },
        SettingSchema { path: "security.enableConseca", description: "Enable Context-Aware Security checker.", default_val: serde_json::Value::Bool(false) },
        SettingSchema { path: "experimental.plan", description: "Enable Plan Mode.", default_val: serde_json::Value::Bool(true) },
    ]
}

pub fn get_infra_list_items<'a>(state: &'a State, view: View) -> Vec<ListItem<'a>> {
    match view {
        View::Settings => {
            let mut last_section = String::new();
            let mut items = Vec::new();
            
            for (path, val, is_default) in get_all_settings(state) {
                let section = path.split('.').next().unwrap_or("").to_uppercase();
                if section != last_section {
                    items.push(ListItem::new(Line::from(format!("─── {} ───", section)).dark_gray()));
                    last_section = section;
                }

                let val_str = match &val {
                    serde_json::Value::Bool(b) => if *b { "󰄬 ON".to_string() } else { "󰄱 OFF".to_string() },
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => if s.is_empty() { "None".to_string() } else { s.clone() },
                    _ => "...".to_string(),
                };

                let key_parts: Vec<_> = path.split('.').collect();
                let key_name = if key_parts.len() > 1 { key_parts[1..].join(".") } else { path.clone() };

                let spans = vec![
                    Span::raw(" "),
                    if is_default { 
                        Span::styled(key_name, Style::default().fg(Color::DarkGray))
                    } else {
                        Span::styled(key_name, Style::default().fg(Color::White).bold())
                    },
                    Span::raw(" "),
                ];

                items.push(ListItem::new(vec![
                    Line::from(spans),
                    Line::from(vec![Span::raw("   "), Span::styled(val_str, Style::default().fg(Color::Cyan))]),
                ]));
            }
            items
        },
        _ => { // Keep existing logic for other views
            get_original_list_items(state, view)
        }
    }
}

pub fn get_all_settings(state: &State) -> Vec<(String, serde_json::Value, bool)> {
    let mut results = Vec::new();
    let current_flat = flatten_settings_helper(&state.settings, "");
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Add current settings from file
    for (path, val) in current_flat {
        results.push((path.clone(), val, false));
        seen_paths.insert(path);
    }

    // 2. Add known settings if missing
    for schema in get_known_settings() {
        if !seen_paths.contains(schema.path) {
            results.push((schema.path.to_string(), schema.default_val.clone(), true));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

fn get_original_list_items<'a>(state: &'a State, view: View) -> Vec<ListItem<'a>> {
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
        _ => Vec::new(),
    }
}

pub fn flatten_settings_helper(v: &serde_json::Value, prefix: &str) -> Vec<(String, serde_json::Value)> {
    let mut items = Vec::new();
    if let Some(obj) = v.as_object() {
        let mut keys: Vec<_> = obj.keys().collect();
        keys.sort();
        for k in keys {
            let new_prefix = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
            let val = &obj[k];
            if val.is_object() && k != "mcpServers" { // Don't flatten MCP servers too deep
                items.extend(flatten_settings_helper(val, &new_prefix));
            } else {
                items.push((new_prefix, val.clone()));
            }
        }
    }
    items
}

pub fn get_setting_at_index(state: &State, target_idx: usize) -> Option<(String, serde_json::Value, bool)> {
    let mut current_idx = 0;
    let mut last_section = String::new();
    
    for (path, val, is_default) in get_all_settings(state) {
        let section = path.split('.').next().unwrap_or("").to_uppercase();
        if section != last_section {
            if current_idx == target_idx { return None; } // Header selected
            current_idx += 1;
            last_section = section;
        }
        if current_idx == target_idx {
            return Some((path, val, is_default));
        }
        current_idx += 1;
    }
    None
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
        View::Memory | View::Plans | View::Health | View::Skills | View::MCP => {
             // ... (Keep existing logic for other views)
             render_original_infra_detail(f, app, area, state, selected);
             return;
        },
        View::Settings => {
            if let Some((path, val, is_default)) = get_setting_at_index(state, selected) {
                title = format!(" Setting: {} ", path);
                markdown = format!("# Setting: {}\n\n", path);
                if is_default {
                    markdown.push_str(&format!("**Status**: [NOT SET - Showing Default]\n\n"));
                }
                markdown.push_str(&format!("**Current Value**: `{}`\n\n", val));
                markdown.push_str("---\n\n### Description\n");
                markdown.push_str(&get_setting_description(&path));
                markdown.push_str("\n\n*Press Enter to edit/set this value*");
            } else {
                markdown = "### Section Header Selected\nSelect a setting below to view details or edit.".to_string();
            }
        },
        _ => {}
    }

    crate::ui::render_markdown(f, app, area, &title, &markdown);
}

fn get_setting_description(path: &str) -> String {
    for schema in get_known_settings() {
        if schema.path == path {
            return schema.description.to_string();
        }
    }
    "No detailed description available for this setting.".to_string()
}

// Helper to keep the file clean since I'm overwriting
fn render_original_infra_detail(f: &mut Frame, app: &App, area: Rect, state: &State, selected: usize) {
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
            markdown = format!("# Health Score: {}\n**Critical**: {} • **Warnings**: {}\n---\n", score, critical, warnings);
            if let Some(rule) = sorted_rules.get(selected) {
                title = format!(" {} ", rule);
                markdown.push_str(&format!("## Rule: {}\n", rule));
                for issue in &groups[*rule] {
                    markdown.push_str(&format!("### {}\n**Project**: {} • **File**: {} • **Severity**: {}\n", 
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
                markdown = format!("# MCP Server: {}\n", s.name);
                if let Some(url) = &s.url { markdown.push_str(&format!("- **Remote URL**: {}\n", url)); }
                if let Some(cmd) = &s.command { markdown.push_str(&format!("- **Local Command**: `{}`\n", cmd)); }
                if !s.args.is_empty() { markdown.push_str(&format!("- **Arguments**: `{}`\n", s.args.join(" "))); }
                if !s.env.is_empty() {
                    markdown.push_str("\n### Environment\n");
                    for (k, v) in &s.env { markdown.push_str(&format!("- **{}**: `{}`\n", k, v)); }
                }
            }
        },
        _ => {}
    }
    crate::ui::render_markdown(f, app, area, &title, &markdown);
}
