use crate::models::*;
use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use std::collections::HashMap;
use entropy::shannon_entropy;
use chrono;

pub struct Parser {
    pub base_dir: PathBuf,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").context("HOME env var not set")?;
        let base_dir = Path::new(&home).join(".gemini").join("tmp");
        Ok(Self { base_dir })
    }

    pub fn discover_projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        
        // 1. Scan global tmp
        if self.base_dir.exists() {
            for entry in fs::read_dir(&self.base_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let chats_dir = path.join("chats");
                    if chats_dir.exists() {
                        if let Ok(sessions) = self.list_sessions(&path) {
                            if sessions.is_empty() { continue; }
                            
                            let (memory_files, plan_files) = self.discover_files(&path);
                            
                            projects.push(Project {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                sessions,
                                memory_files,
                                plan_files,
                            });
                        }
                    }
                }
            }
        }

        // 2. Fallback: Check if current dir is a project
        let curr_dir = std::env::current_dir()?;
        if curr_dir.join("chats").exists() {
             if let Ok(sessions) = self.list_sessions(&curr_dir) {
                if !sessions.is_empty() && !projects.iter().any(|p| p.path == curr_dir.to_string_lossy().to_string()) {
                    let (memory_files, plan_files) = self.discover_files(&curr_dir);
                    projects.push(Project {
                        name: "Current Project".to_string(),
                        path: curr_dir.to_string_lossy().to_string(),
                        sessions,
                        memory_files,
                        plan_files,
                    });
                }
             }
        }
        
        projects.sort_by(|a, b| {
            let a_last = a.sessions.first().map(|s| s.last_updated);
            let b_last = b.sessions.first().map(|s| s.last_updated);
            b_last.cmp(&a_last)
        });

        Ok(projects)
    }

    fn discover_files(&self, project_tmp_path: &Path) -> (Vec<ProjectFile>, Vec<ProjectFile>) {
        let mut memories = Vec::new();
        let mut plans = Vec::new();

        // 1. Check for .project_root to find actual workspace
        if let Ok(root_path_str) = fs::read_to_string(project_tmp_path.join(".project_root")) {
            let workspace_root = Path::new(root_path_str.trim());
            let gemini_md = workspace_root.join("GEMINI.md");
            if gemini_md.exists() {
                memories.push(ProjectFile {
                    name: "Project GEMINI.md".to_string(),
                    path: gemini_md.to_string_lossy().to_string(),
                    category: "Memory".to_string(),
                });
            }
        }

        // 2. Global Memory
        if let Ok(home) = std::env::var("HOME") {
            let global_md = Path::new(&home).join(".gemini").join("GEMINI.md");
            if global_md.exists() {
                memories.push(ProjectFile {
                    name: "Global GEMINI.md".to_string(),
                    path: global_md.to_string_lossy().to_string(),
                    category: "Memory".to_string(),
                });
            }
        }

        // 3. Plans in tmp dir
        let plans_dir = project_tmp_path.join("plans");
        if plans_dir.exists() {
            if let Ok(entries) = fs::read_dir(plans_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().map_or(false, |e| e == "md") {
                        plans.push(ProjectFile {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                            category: "Plan".to_string(),
                        });
                    }
                }
            }
        }

        (memories, plans)
    }

    pub fn list_sessions(&self, project_path: &Path) -> Result<Vec<Session>> {
        let chats_dir = project_path.join("chats");
        let mut sessions = Vec::new();
        if !chats_dir.exists() { return Ok(sessions); }

        for entry in fs::read_dir(chats_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(session) = self.parse_session(&path) {
                    sessions.push(session);
                }
            }
        }
        sessions.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
        Ok(sessions)
    }

    pub fn parse_session(&self, path: &Path) -> Result<Session> {
        let data = fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&data)?;
        Ok(session)
    }

    pub fn get_full_state(&self) -> Result<State> {
        let projects = self.discover_projects()?;
        let mut all_sessions = Vec::new();
        let mut timeline = Vec::new();
        let mut health = Vec::new();
        
        let mcp_servers = self.discover_mcp_servers().unwrap_or_default();
        let skills = self.discover_skills().unwrap_or_default();
        let settings = self.parse_settings().unwrap_or_default();

        let secret_regexes = vec![
            ("Private Key", Regex::new(r"-----BEGIN [a-zA-Z\s]+ PRIVATE KEY-----").unwrap(), 0),
            ("AWS Access Key", Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").unwrap(), 1),
            ("Auth Header", Regex::new(r"(?i)Authorization:\s*(?:Bearer|Basic)\s+([a-zA-Z0-9_=\-\.]{16,})").unwrap(), 1),
            ("Platform Token", Regex::new(r"(ghp_|xox[bap]-|npm_|rk_live_|sk_live_|AIzaSy)[a-zA-Z0-9_\-]{20,}").unwrap(), 0),
            ("API Key/Secret", Regex::new(r"(?i)(?:api_key|apikey|secret|token|password)[\s:=]+[\x22\x27]?([a-zA-Z0-9_\-\.]{16,})[\x22\x27]?").unwrap(), 1),
            ("DB Connection", Regex::new(r"(?i)(?:postgres|mysql|redis|mongodb)://[a-zA-Z0-9_-]+:([a-zA-Z0-9_\-\.!@#$%^&*]+)@").unwrap(), 1),
        ];

        let now = chrono::Utc::now();
        let mut ses_count = 0;

        for proj in &projects {
            // 1. Project Health Rules
            if proj.memory_files.is_empty() {
                health.push(HealthIssue {
                    id: "CFG001".to_string(),
                    severity: "Warning".to_string(),
                    message: "Missing project GEMINI.md".to_string(),
                    category: "Config".to_string(),
                    project: proj.name.clone(),
                    file: None,
                    rule: "Project Context".to_string(),
                });
            } else {
                for f in &proj.memory_files {
                    if let Ok(content) = fs::read_to_string(&f.path) {
                        if content.len() < 100 {
                            health.push(HealthIssue {
                                id: "CFG002".to_string(),
                                severity: "Info".to_string(),
                                message: "GEMINI.md is very short; consider adding more project context.".to_string(),
                                category: "Config".to_string(),
                                project: proj.name.clone(),
                                file: Some(f.name.clone()),
                                rule: "Context Depth".to_string(),
                            });
                        }
                    }
                }
            }

            for sess in &proj.sessions {
                all_sessions.push(sess.clone());
                timeline.push(TimelineEvent {
                    session: sess.clone(),
                    project: proj.name.clone(),
                });

                let mut session_cost = 0.0;
                let mut session_tokens = 0;
                let msg_count = sess.messages.len();

                // 2. Secret Scanning & Accumulate Stats
                for msg in &sess.messages {
                    if let (Some(model), Some(tokens)) = (&msg.model, &msg.tokens) {
                        let pricing = get_pricing(model);
                        session_cost += (tokens.input as f64 / 1_000_000.0 * pricing.0) + (tokens.output as f64 / 1_000_000.0 * pricing.1);
                        session_tokens += tokens.total;
                    }

                    let text = format_value(&msg.content);
                    for (name, re, capture_group) in &secret_regexes {
                        for cap in re.captures_iter(&text) {
                            let match_str = cap.get(*capture_group).map_or("", |m| m.as_str());
                            // Entropy check to filter false positives like Git SHAs
                            if shannon_entropy(match_str.as_bytes()) > 3.5 || *capture_group == 0 {
                                health.push(HealthIssue {
                                    id: "SEC001".to_string(),
                                    severity: "Critical".to_string(),
                                    message: format!("Leaked {} detected in session history.", name),
                                    category: "Security".to_string(),
                                    project: proj.name.clone(),
                                    file: Some(sess.session_id.clone()),
                                    rule: "Secret Leakage".to_string(),
                                });
                            }
                        }
                    }
                }

                // 3. Session Health Rules (SES)
                if ses_count < 10 {
                    let idle_days = (now - sess.last_updated).num_days();
                    
                    if session_cost > 25.0 {
                        health.push(HealthIssue { id: "SES001".to_string(), severity: "Warning".to_string(), message: format!("Session cost exceeded $25 (${:.2})", session_cost), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Cost Limit".to_string() });
                        ses_count += 1;
                    } else if msg_count > 200 {
                        health.push(HealthIssue { id: "SES002".to_string(), severity: "Warning".to_string(), message: format!("Conversation exceeded 200 messages ({})", msg_count), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Message Limit".to_string() });
                        ses_count += 1;
                    } else if session_tokens > 5_000_000 {
                        health.push(HealthIssue { id: "SES003".to_string(), severity: "Warning".to_string(), message: format!("Token consumption exceeded 5M ({})", session_tokens), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Token Limit".to_string() });
                        ses_count += 1;
                    } else if idle_days > 7 && msg_count > 50 {
                        health.push(HealthIssue { id: "SES004".to_string(), severity: "Info".to_string(), message: format!("Session idle for {} days with {} msgs", idle_days, msg_count), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Stale Session".to_string() });
                        ses_count += 1;
                    }
                }
            }
        }

        // 4. Skill Health Rules
        for skill in &skills {
            if skill.description.len() < 10 {
                health.push(HealthIssue {
                    id: "SKL001".to_string(),
                    severity: "Warning".to_string(),
                    message: format!("Skill '{}' has a very short description.", skill.name),
                    category: "Config".to_string(),
                    project: "Global".to_string(),
                    file: Some(skill.path.clone()),
                    rule: "Skill Documentation".to_string(),
                });
            }
        }

        timeline.sort_by(|a, b| b.session.last_updated.cmp(&a.session.last_updated));

        Ok(State {
            projects: projects.clone(),
            all_sessions,
            stats: self.calculate_stats(&projects),
            health,
            timeline,
            mcp_servers,
            skills,
            settings,
        })
    }

    pub fn parse_settings(&self) -> Result<serde_json::Value> {
        let home = std::env::var("HOME")?;
        let settings_path = Path::new(&home).join(".gemini").join("settings.json");
        if settings_path.exists() {
            let data = fs::read_to_string(settings_path)?;
            let v: serde_json::Value = serde_json::from_str(&data)?;
            return Ok(v);
        }
        Ok(serde_json::json!({}))
    }

    pub fn discover_mcp_servers(&self) -> Result<Vec<McpServer>> {
        let home = std::env::var("HOME")?;
        let settings_path = Path::new(&home).join(".gemini").join("settings.json");
        if !settings_path.exists() { return Ok(vec![]); }

        let data = fs::read_to_string(settings_path)?;
        let v: serde_json::Value = serde_json::from_str(&data)?;
        let mut servers = Vec::new();

        if let Some(mcp) = v.get("mcpServers").and_then(|m| m.as_object()) {
            for (name, config) in mcp {
                let mut server = McpServer {
                    name: name.clone(),
                    url: config.get("httpUrl").and_then(|u| u.as_str()).map(|s| s.to_string()),
                    command: config.get("command").and_then(|u| u.as_str()).map(|s| s.to_string()),
                    args: vec![],
                    env: HashMap::new(),
                };
                if let Some(args) = config.get("args").and_then(|a| a.as_array()) {
                    server.args = args.iter().filter_map(|a| a.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(env) = config.get("env").and_then(|e| e.as_object()) {
                    for (k, val) in env {
                        if let Some(s) = val.as_str() {
                            server.env.insert(k.clone(), s.to_string());
                        }
                    }
                }
                servers.push(server);
            }
        }
        Ok(servers)
    }

    pub fn discover_skills(&self) -> Result<Vec<Skill>> {
        let home = std::env::var("HOME")?;
        let ext_dir = Path::new(&home).join(".gemini").join("extensions");
        let mut skills = Vec::new();
        if !ext_dir.exists() { return Ok(skills); }

        for entry in fs::read_dir(ext_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let ext_name = entry.file_name().to_string_lossy().to_string();
                let prompts_dir = path.join("commands").join("prompts");
                if prompts_dir.exists() {
                    for p_entry in fs::read_dir(prompts_dir)? {
                        let p_entry = p_entry?;
                        let p_path = p_entry.path();
                        if p_path.extension().map_or(false, |e| e == "toml") {
                            if let Ok(content) = fs::read_to_string(&p_path) {
                                // Basic parsing for description and prompt
                                let desc = content.lines()
                                    .find(|l| l.starts_with("description ="))
                                    .and_then(|l| l.split('\"').nth(1))
                                    .unwrap_or("No description")
                                    .to_string();
                                
                                skills.push(Skill {
                                    name: p_path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                                    extension: ext_name.clone(),
                                    description: desc,
                                    args_description: None,
                                    path: p_path.to_string_lossy().to_string(),
                                    content,
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(skills)
    }

    pub fn calculate_stats(&self, projects: &[Project]) -> GlobalStats {
        let mut gs = GlobalStats {
            projects: HashMap::new(),
            overall: ProjectStats::default(),
        };
        gs.overall.name = "Overall".to_string();

        for proj in projects {
            let mut ps = ProjectStats::default();
            ps.name = proj.name.clone();

            for sess in &proj.sessions {
                for msg in &sess.messages {
                    let model = msg.model.as_deref().unwrap_or("gemini-1.5-pro");
                    let pricing = get_pricing(model);
                    
                    if let Some(tokens) = &msg.tokens {
                        let cost = (tokens.input as f64 / 1_000_000.0 * pricing.0) +
                                   (tokens.output as f64 / 1_000_000.0 * pricing.1);
                        
                        ps.cost += cost;
                        ps.total_tokens += tokens.total;
                        ps.input += tokens.input;
                        ps.output += tokens.output;
                        *ps.models.entry(model.to_string()).or_insert(0) += 1;
                        ps.token_history.push((msg.timestamp, tokens.total));

                        gs.overall.cost += cost;
                        gs.overall.total_tokens += tokens.total;
                        gs.overall.input += tokens.input;
                        gs.overall.output += tokens.output;
                        *gs.overall.models.entry(model.to_string()).or_insert(0) += 1;
                        gs.overall.token_history.push((msg.timestamp, tokens.total));
                    }
                }
            }
            ps.token_history.sort_by_key(|h| h.0);
            gs.projects.insert(proj.name.clone(), ps);
        }
        gs.overall.token_history.sort_by_key(|h| h.0);
        gs
    }
}

fn get_pricing(model: &str) -> (f64, f64) {
    if model.contains("flash") { (0.075, 0.30) }
    else { (3.50, 10.50) }
}

fn format_value(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    format!("{}", content)
}
