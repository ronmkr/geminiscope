use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub projects: Vec<Project>,
    pub all_sessions: Vec<Session>,
    pub stats: GlobalStats,
    pub health: Vec<HealthIssue>,
    pub timeline: Vec<TimelineEvent>,
    pub mcp_servers: Vec<McpServer>,
    pub skills: Vec<Skill>,
    pub settings: serde_json::Value,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub sidebar_bg: String,
    pub text: String,
    pub json_key: String,
    pub json_value: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: "Magenta".to_string(),
            secondary: "Cyan".to_string(),
            accent: "Yellow".to_string(),
            sidebar_bg: "#313244".to_string(),
            text: "#CDD6F4".to_string(),
            json_key: "Cyan".to_string(),
            json_value: "Yellow".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub extension: String,
    pub description: String,
    pub args_description: Option<String>,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub sessions: Vec<Session>,
    pub memory_files: Vec<ProjectFile>,
    pub plan_files: Vec<ProjectFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub name: String,
    pub path: String,
    pub category: String, // "Memory" or "Plan"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub project_hash: String,
    pub start_time: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub content: serde_json::Value,
    pub thoughts: Option<Vec<Thought>>,
    pub tokens: Option<Tokens>,
    pub model: Option<String>,
    #[serde(rename = "toolCalls")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub status: String,
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub subject: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub input: i64,
    pub output: i64,
    pub cached: i64,
    pub thoughts: i64,
    pub tool: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub projects: HashMap<String, ProjectStats>,
    pub overall: ProjectStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStats {
    pub name: String,
    pub cost: f64,
    pub total_tokens: i64,
    pub models: HashMap<String, i64>,
    pub input: i64,
    pub output: i64,
    pub token_history: Vec<(DateTime<Utc>, i64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub category: String,
    pub project: String,
    pub file: Option<String>,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub session: Session,
    pub project: String,
}
