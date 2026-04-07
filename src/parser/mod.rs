pub mod session;
pub mod mcp;
pub mod stats;
pub mod skills;
pub mod health;
pub mod project;
pub mod config;

use crate::models::*;
use anyhow::{Result, Context};

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

    pub fn get_full_state(&self) -> Result<State> {
        let projects = project::discover_projects(&self.base_dir, &self.session_cache)?;
        let mut all_sessions = Vec::new();
        let mut timeline = Vec::new();
        let mut health = Vec::new();
        
        let mcp_servers = mcp::discover_mcp_servers().unwrap_or_default();
        let skills = skills::discover_skills().unwrap_or_default();
        let settings = config::parse_settings().unwrap_or_default();
        let theme = config::parse_theme().unwrap_or_default();

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
            theme,
        })
    }

    pub fn save_settings(&self, settings: &serde_json::Value) -> Result<()> {
        config::save_settings(settings)
    }
}
