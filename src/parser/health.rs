use crate::models::{Project, Session, HealthIssue, Skill, format_value};
use entropy::shannon_entropy;
use crate::parser::stats::get_pricing;
use std::fs;

use crate::parser::security;

pub struct HealthChecker {}

impl HealthChecker {
    pub fn check_project_health(proj: &Project, health: &mut Vec<HealthIssue>) {
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
                if let Ok(content) = fs::read_to_string(&f.path)
                    && content.len() < 100 {
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

    pub fn check_session_health(proj: &Project, sess: &Session, health: &mut Vec<HealthIssue>, now: chrono::DateTime<chrono::Utc>, ses_count: &mut usize) {
        let mut session_cost = 0.0;
        let mut session_tokens = 0;
        let msg_count = sess.messages.len();

        for msg in &sess.messages {
            if let (Some(model), Some(tokens)) = (&msg.model, &msg.tokens) {
                let pricing = get_pricing(model);
                session_cost += (tokens.input as f64 / 1_000_000.0 * pricing.0) + (tokens.output as f64 / 1_000_000.0 * pricing.1);
                session_tokens += tokens.total;
            }

            let text = format_value(&msg.content);
            for pattern in security::get_secret_patterns() {
                for cap in pattern.regex.captures_iter(&text) {
                    let match_str = cap.get(pattern.capture_group).map_or("", |m| m.as_str());
                    if shannon_entropy(match_str.as_bytes()) > 3.5 || pattern.capture_group == 0 {
                        health.push(HealthIssue {
                            id: "SEC001".to_string(),
                            severity: "Critical".to_string(),
                            message: format!("Leaked {} detected in session history.", pattern.name),
                            category: "Security".to_string(),
                            project: proj.name.clone(),
                            file: Some(sess.session_id.clone()),
                            rule: "Secret Leakage".to_string(),
                        });
                    }
                }
            }
        }

        if *ses_count < 10 {
            let idle_days = (now - sess.last_updated).num_days();
            
            if session_cost > 25.0 {
                health.push(HealthIssue { id: "SES001".to_string(), severity: "Warning".to_string(), message: format!("Session cost exceeded $25 (${session_cost:.2})"), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Cost Limit".to_string() });
                *ses_count += 1;
            } else if msg_count > 200 {
                health.push(HealthIssue { id: "SES002".to_string(), severity: "Warning".to_string(), message: format!("Conversation exceeded 200 messages ({msg_count})"), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Message Limit".to_string() });
                *ses_count += 1;
            } else if session_tokens > 5_000_000 {
                health.push(HealthIssue { id: "SES003".to_string(), severity: "Warning".to_string(), message: format!("Token consumption exceeded 5M ({session_tokens})"), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Token Limit".to_string() });
                *ses_count += 1;
            } else if idle_days > 7 && msg_count > 50 {
                health.push(HealthIssue { id: "SES004".to_string(), severity: "Info".to_string(), message: format!("Session idle for {idle_days} days with {msg_count} msgs"), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Stale Session".to_string() });
                *ses_count += 1;
            }
        }
    }

    pub fn check_skill_health(skill: &Skill, health: &mut Vec<HealthIssue>) {
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
}
