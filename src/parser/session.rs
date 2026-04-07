use crate::models::*;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn list_sessions(project_path: &Path) -> Result<Vec<Session>> {
    let chats_dir = project_path.join("chats");
    let mut sessions = Vec::new();
    if !chats_dir.exists() { return Ok(sessions); }

    for entry in fs::read_dir(chats_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(session) = parse_session(&path) {
                sessions.push(session);
            }
        }
    }
    sessions.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
    Ok(sessions)
}

pub fn parse_session(path: &Path) -> Result<Session> {
    let data = fs::read_to_string(path)?;
    let session: Session = serde_json::from_str(&data)?;
    Ok(session)
}
