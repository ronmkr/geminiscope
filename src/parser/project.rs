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
    
    let home = std::env::var("HOME").unwrap_or_default();
    let home_path = Path::new(&home);

    // 1. Scan global tmp
    if base_dir.exists() {
        for entry in fs::read_dir(base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Canonicalize and ensure it's within HOME
                let canon_path = match path.canonicalize() {
                    Ok(p) if p.starts_with(home_path) => p,
                    _ => continue,
                };

                let chats_dir = canon_path.join("chats");
                if chats_dir.exists()
                    && let Ok(sessions) = session::list_sessions(&canon_path, session_cache) {
                        if sessions.is_empty() { continue; }
                        
                        let (memory_files, plan_files) = discover_files(&canon_path);
                        
                        projects.push(Project {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: canon_path.to_string_lossy().to_string(),
                            sessions,
                            memory_files,
                            plan_files,
                        });
                    }
            }
        }
    }

    // 2. Fallback: Check if current dir is a project
    if let Ok(curr_dir) = std::env::current_dir()
        && let Ok(canon_curr) = curr_dir.canonicalize()
            && canon_curr.starts_with(home_path) && canon_curr.join("chats").exists()
                 && let Ok(sessions) = session::list_sessions(&canon_curr, session_cache)
                    && !sessions.is_empty() && !projects.iter().any(|p| p.path == canon_curr.to_string_lossy()) {
                        let (memory_files, plan_files) = discover_files(&canon_curr);
                        projects.push(Project {
                            name: "Current Project".to_string(),
                            path: canon_curr.to_string_lossy().to_string(),
                            sessions,
                            memory_files,
                            plan_files,
                        });
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
        let home = std::env::var("HOME").unwrap_or_default();
        let home_path = Path::new(&home);

        // Basic security check: Ensure workspace is within HOME to prevent arbitrary traversal
        if workspace_root.starts_with(home_path) && workspace_root.exists() {
            let gemini_md = workspace_root.join("GEMINI.md");
            if gemini_md.exists() {
                memories.push(ProjectFile {
                    name: "Project GEMINI.md".to_string(),
                    path: gemini_md.to_string_lossy().to_string(),
                    category: "Memory".to_string(),
                });
            }
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
    if plans_dir.exists()
        && let Ok(entries) = fs::read_dir(plans_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "md") {
                    plans.push(ProjectFile {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path().to_string_lossy().to_string(),
                        category: "Plan".to_string(),
                    });
                }
            }
        }

    (memories, plans)
}
