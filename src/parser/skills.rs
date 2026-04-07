use crate::models::*;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn discover_skills() -> Result<Vec<Skill>> {
    let home = std::env::var("HOME")?;
    let ext_dir = Path::new(&home).join(".gemini").join("extensions");
    let mut skills = Vec::new();
    if !ext_dir.exists() { return Ok(skills); }

    for entry in fs::read_dir(ext_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let ext_name = entry.file_name().to_string_lossy().to_string();
            let prompts_dir = path.join("commands").join("prompts");
            if prompts_dir.exists() {
                for p_entry in fs::read_dir(prompts_dir)? {
                    let p_entry = p_entry?;
                    let p_path = p_entry.path();
                    if p_path.extension().map_or(false, |e| e == "toml") {
                        if let Ok(content) = fs::read_to_string(&p_path) {
                            // Basic parsing for description and prompt
                            let desc = content.lines()
                                .find(|l| l.starts_with("description ="))
                                .and_then(|l| l.split('\"').nth(1))
                                .unwrap_or("No description")
                                .to_string();
                            
                            skills.push(Skill {
                                name: p_path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                                extension: ext_name.clone(),
                                description: desc,
                                args_description: None,
                                path: p_path.to_string_lossy().to_string(),
                                content,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(skills)
}
