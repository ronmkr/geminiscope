use crate::models::*;
use regex::Regex;
use entropy::shannon_entropy;
use crate::parser::stats::get_pricing;
use std::fs;

pub struct HealthChecker {
    secret_regexes: Vec<(&'static str, Regex, usize)>,
}

impl HealthChecker {
    pub fn new() -> Self {
        let secret_regexes = vec![
            ("Private Key", Regex::new(r"-----BEGIN [a-zA-Z\s]+ PRIVATE KEY-----").expect("Static Regex"), 0),
            ("AWS Access Key", Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").expect("Static Regex"), 1),
            ("Auth Header", Regex::new(r"(?i)Authorization:\s*(?:Bearer|Basic)\s+([a-zA-Z0-9_=\-\.]{16,})").expect("Static Regex"), 1),
            ("Platform Token", Regex::new(r"(ghp_|xox[bap]-|npm_|rk_live_|sk_live_|AIzaSy)[a-zA-Z0-9_\-]{20,}").expect("Static Regex"), 0),
            ("API Key/Secret", Regex::new(r"(?i)(?:api_key|apikey|secret|token|password)[\s:=]+[\x22\x27]?([a-zA-Z0-9_\-\.]{16,})[\x22\x27]?").expect("Static Regex"), 1),
            ("DB Connection", Regex::new(r"(?i)(?:postgres|mysql|redis|mongodb)://[a-zA-Z0-9_-]+:([a-zA-Z0-9_\-\.!@#$%^&*]+)@").expect("Static Regex"), 1),
        ];
        Self { secret_regexes }
    }

    pub fn check_project_health(&self, proj: &Project, health: &mut Vec<HealthIssue>) {
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
    }

    pub fn check_session_health(&self, proj: &Project, sess: &Session, health: &mut Vec<HealthIssue>, now: chrono::DateTime<chrono::Utc>, ses_count: &mut usize) {
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
            for (name, re, capture_group) in &self.secret_regexes {
                for cap in re.captures_iter(&text) {
                    let match_str = cap.get(*capture_group).map_or("", |m| m.as_str());
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

        if *ses_count < 10 {
            let idle_days = (now - sess.last_updated).num_days();
            
            if session_cost > 25.0 {
                health.push(HealthIssue { id: "SES001".to_string(), severity: "Warning".to_string(), message: format!("Session cost exceeded $25 (${:.2})", session_cost), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Cost Limit".to_string() });
                *ses_count += 1;
            } else if msg_count > 200 {
                health.push(HealthIssue { id: "SES002".to_string(), severity: "Warning".to_string(), message: format!("Conversation exceeded 200 messages ({})", msg_count), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Message Limit".to_string() });
                *ses_count += 1;
            } else if session_tokens > 5_000_000 {
                health.push(HealthIssue { id: "SES003".to_string(), severity: "Warning".to_string(), message: format!("Token consumption exceeded 5M ({})", session_tokens), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Token Limit".to_string() });
                *ses_count += 1;
            } else if idle_days > 7 && msg_count > 50 {
                health.push(HealthIssue { id: "SES004".to_string(), severity: "Info".to_string(), message: format!("Session idle for {} days with {} msgs", idle_days, msg_count), category: "Performance".to_string(), project: proj.name.clone(), file: Some(sess.session_id.clone()), rule: "Stale Session".to_string() });
                *ses_count += 1;
            }
        }
    }

    pub fn check_skill_health(&self, skill: &Skill, health: &mut Vec<HealthIssue>) {
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

fn format_value(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    format!("{}", content)
}
