use std::process::Command;
use anyhow::Result;

pub fn open_in_editor(editor: &str, file_path: &str) -> Result<()> {
    // Basic validation to ensure the editor string isn't obviously malicious
    if editor.is_empty() || editor.contains(';') || editor.contains('&') || editor.contains('|') {
        return Err(anyhow::anyhow!("Invalid editor command"));
    }

    // We avoid Command::new("sh").arg("-c")... to prevent shell injection.
    // Instead we spawn the editor process directly.
    Command::new(editor)
        .arg(file_path)
        .status()?;
    
    Ok(())
}
