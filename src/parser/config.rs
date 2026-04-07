use crate::models::Theme;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn parse_theme() -> Result<Theme> {
    let home = std::env::var("HOME")?;
    let gemini_dir = Path::new(&home).join(".gemini");
    let theme_file = gemini_dir.join("themes.json");
    let settings = parse_settings().unwrap_or_default();
    let theme_name = settings.get("ui").and_then(|ui| ui.get("theme")).and_then(|t| t.as_str()).unwrap_or("Default");

    if theme_file.exists() {
        let data = fs::read_to_string(&theme_file)?;
        let themes: serde_json::Value = serde_json::from_str(&data)?;
        if let Some(t_val) = themes.get(theme_name)
            && let Ok(theme) = serde_json::from_value(t_val.clone()) {
                return Ok(theme);
            }
    }

    // Fallback to old path if themes.json doesn't exist or doesn't have the theme
    let legacy_path = gemini_dir.join("geminiscope_theme.json");
    if legacy_path.exists() {
        let data = fs::read_to_string(legacy_path)?;
        if let Ok(theme) = serde_json::from_str(&data) {
            return Ok(theme);
        }
    }

    Ok(Theme::default())
}

pub fn parse_settings() -> Result<serde_json::Value> {
    let home = std::env::var("HOME")?;
    let settings_path = Path::new(&home).join(".gemini").join("settings.json");
    if settings_path.exists() {
        let data = fs::read_to_string(settings_path)?;
        let v: serde_json::Value = serde_json::from_str(&data)?;
        return Ok(v);
    }
    Ok(serde_json::json!({}))
}

pub fn save_settings(settings: &serde_json::Value) -> Result<()> {
    let home = std::env::var("HOME")?;
    let settings_path = Path::new(&home).join(".gemini").join("settings.json");
    let temp_path = settings_path.with_extension("tmp");
    let data = serde_json::to_string_pretty(settings)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);
        let mut file = options.open(&temp_path)?;
        file.write_all(data.as_bytes())?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&temp_path, data)?;
    }

    fs::rename(temp_path, settings_path)?;
    Ok(())
}
