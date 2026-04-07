use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum View {
    Chats,
    Stats,
    Tools,
    Memory,
    Plans,
    Health,
    Timeline,
    Skills,
    MCP,
    Settings,
    Diff,
}

impl View {
    pub fn title(&self) -> &str {
        match self {
            Self::Chats => " CHATS ",
            Self::Stats => " PROJECTS ",
            Self::Tools => " TOOLS ",
            Self::Memory => " MEMORY ",
            Self::Plans => " PLANS ",
            Self::Health => " HEALTH ",
            Self::Timeline => " TIMELINE ",
            Self::Skills => " SKILLS ",
            Self::MCP => " MCPS ",
            Self::Settings => " SETTINGS ",
            Self::Diff => " DIFF ",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Chats => "󰭻",
            Self::Stats => "󰄦",
            Self::Tools => "󰓙",
            Self::Memory => "󰤄",
            Self::Plans => "󰏚",
            Self::Health => "󰓚",
            Self::Timeline => "󰃭",
            Self::Skills => "󰛨",
            Self::MCP => "󰒄",
            Self::Settings => "󰒓",
            Self::Diff => "󰒺",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Chats, Self::Stats, Self::Tools, Self::Memory, Self::Plans,
            Self::Health, Self::Timeline, Self::Skills, Self::MCP, Self::Settings, Self::Diff
        ]
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum ProjectSort {
    Date,
    Cost,
    Tokens,
    Name,
}

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

impl State {
    pub fn filtered_sessions(&self, query: &str) -> Vec<&Session> {
        if query.is_empty() {
            self.all_sessions.iter().collect()
        } else {
            let q = query.to_lowercase();
            self.all_sessions.iter()
                .filter(|s| s.search_text().to_lowercase().contains(&q))
                .collect()
        }
    }
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

impl Session {
    pub fn search_text(&self) -> String {
        self.messages.iter().map(|m| m.search_text()).collect::<Vec<_>>().join(" ")
    }

    pub fn full_text(&self) -> String {
        let mut text = String::new();
        for msg in &self.messages {
            let header = if msg.msg_type == "user" { "USER" } else { "GEMINI" };
            text.push_str(&format!("### {}\n", header));
            text.push_str(&msg.raw_content());
            text.push_str("\n\n");
        }
        text
    }
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

impl Message {
    pub fn raw_content(&self) -> String {
        format_value(&self.content)
    }

    pub fn search_text(&self) -> String {
        self.raw_content()
    }
}

pub fn format_value(content: &serde_json::Value) -> String {
    if let Some(s) = content.as_str() { return s.to_string(); }
    if let Some(arr) = content.as_array() {
        return arr.iter().filter_map(|v| v.get("text").and_then(|t| t.as_str())).collect::<Vec<_>>().join("");
    }
    format!("{}", content)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&json!("hello")), "hello");
        assert_eq!(format_value(&json!([{"text": "foo"}, {"text": "bar"}])), "foobar");
        assert_eq!(format_value(&json!(123)), "123");
    }

    #[test]
    fn test_session_search_text() {
        let session = Session {
            session_id: "1".to_string(),
            project_hash: "h".to_string(),
            start_time: Utc::now(),
            last_updated: Utc::now(),
            messages: vec![
                Message {
                    id: "m1".to_string(),
                    timestamp: Utc::now(),
                    msg_type: "user".to_string(),
                    content: json!("hello world"),
                    thoughts: None,
                    tokens: None,
                    model: None,
                    tool_calls: None,
                }
            ],
        };
        assert!(session.search_text().contains("hello world"));
    }
}
