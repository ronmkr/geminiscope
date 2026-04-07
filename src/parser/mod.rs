pub mod session;
pub mod mcp;
pub mod stats;
pub mod skills;
pub mod health;

use crate::models::*;
use anyhow::{Result, Context};

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;
use crate::parser::health::HealthChecker;

pub struct Parser {
    pub base_dir: PathBuf,
    session_cache: Mutex<HashMap<PathBuf, (SystemTime, Session)>>,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").context("HOME env var not set")?;
        let base_dir = Path::new(&home).join(".gemini").join("tmp");
        Ok(Self { 
            base_dir,
            session_cache: Mutex::new(HashMap::new()),
        })
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
                        if let Ok(sessions) = session::list_sessions(&path, &self.session_cache) {
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
             if let Ok(sessions) = session::list_sessions(&curr_dir, &self.session_cache) {
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

    pub fn get_full_state(&self) -> Result<State> {
        let projects = self.discover_projects()?;
        let mut all_sessions = Vec::new();
        let mut timeline = Vec::new();
        let mut health = Vec::new();
        
        let mcp_servers = mcp::discover_mcp_servers().unwrap_or_default();
        let skills = skills::discover_skills().unwrap_or_default();
        let settings = self.parse_settings().unwrap_or_default();

        let checker = HealthChecker::new();
        let now = chrono::Utc::now();
        let mut ses_count = 0;

        for proj in &projects {
            checker.check_project_health(proj, &mut health);

            for sess in &proj.sessions {
                all_sessions.push(sess.clone());
                timeline.push(TimelineEvent {
                    session: sess.clone(),
                    project: proj.name.clone(),
                });

                checker.check_session_health(proj, sess, &mut health, now, &mut ses_count);
            }
        }

        for skill in &skills {
            checker.check_skill_health(skill, &mut health);
        }

        timeline.sort_by(|a, b| b.session.last_updated.cmp(&a.session.last_updated));

        Ok(State {
            projects: projects.clone(),
            all_sessions,
            stats: stats::calculate_stats(&projects),
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
}
