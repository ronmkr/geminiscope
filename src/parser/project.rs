use crate::models::{Project, ProjectFile, Session};
use crate::parser::session;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

pub fn discover_projects(
    base_dir: &Path,
    session_cache: &Mutex<HashMap<PathBuf, (SystemTime, Session)>>
) -> Result<Vec<Project>> {
    let mut projects = Vec::new();
    
    // 1. Scan global tmp
    if base_dir.exists() {
        for entry in fs::read_dir(base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let chats_dir = path.join("chats");
                if chats_dir.exists() {
                    if let Ok(sessions) = session::list_sessions(&path, session_cache) {
                        if sessions.is_empty() { continue; }
                        
                        let (memory_files, plan_files) = discover_files(&path);
                        
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
         if let Ok(sessions) = session::list_sessions(&curr_dir, session_cache) {
            if !sessions.is_empty() && !projects.iter().any(|p| p.path == curr_dir.to_string_lossy().to_string()) {
                let (memory_files, plan_files) = discover_files(&curr_dir);
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

pub fn discover_files(project_tmp_path: &Path) -> (Vec<ProjectFile>, Vec<ProjectFile>) {
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
