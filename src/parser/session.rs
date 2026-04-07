use crate::models::*;
use anyhow::{Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::SystemTime;
use std::sync::Mutex;

pub fn list_sessions(
    project_path: &Path, 
    cache: &Mutex<HashMap<PathBuf, (SystemTime, Session)>>
) -> Result<Vec<Session>> {
    let chats_dir = project_path.join("chats");
    let mut sessions = Vec::new();
    if !chats_dir.exists() { return Ok(sessions); }

    let mut cache_lock = cache.lock().unwrap();

    for entry in fs::read_dir(chats_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let metadata = fs::metadata(&path)?;
            let mtime = metadata.modified()?;

            // Check cache
            if let Some((cached_mtime, session)) = cache_lock.get(&path) {
                if *cached_mtime == mtime {
                    sessions.push(session.clone());
                    continue;
                }
            }

            // Parse and update cache
            if let Ok(session) = parse_session(&path) {
                cache_lock.insert(path.clone(), (mtime, session.clone()));
                sessions.push(session);
            }
        }
    }
    
    // Cleanup cache for deleted files in this project (optional but good for long-running)
    // For now, simple caching is enough.
    
    sessions.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
    Ok(sessions)
}

pub fn parse_session(path: &Path) -> Result<Session> {
    let data = fs::read_to_string(path)?;
    let session: Session = serde_json::from_str(&data)?;
    Ok(session)
}
