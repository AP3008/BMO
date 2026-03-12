use crate::config::{BmoConfig, NotesMode};
use std::fs;
use std::path::PathBuf;

/// Resolve the notes directory based on config.
pub fn notes_dir(config: &BmoConfig) -> Result<PathBuf, String> {
    let dir = match config.notes.mode {
        NotesMode::Obsidian => {
            let vault = config
                .notes
                .obsidian_vault_path
                .as_deref()
                .ok_or("Obsidian vault path not configured. Run `bmo --settings`.")?;
            PathBuf::from(vault)
        }
        NotesMode::Local => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".bmo").join("notes")
        }
    };
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create notes dir {}: {}", dir.display(), e))?;
    Ok(dir)
}

/// Read `_memory.md` from the notes dir. Returns None if file doesn't exist.
pub fn get_memory(config: &BmoConfig) -> Option<String> {
    let dir = notes_dir(config).ok()?;
    let path = dir.join("_memory.md");
    fs::read_to_string(&path).ok()
}

/// Write `_memory.md` to the notes dir.
pub fn save_memory(config: &BmoConfig, content: &str) -> Result<(), String> {
    let dir = notes_dir(config)?;
    let path = dir.join("_memory.md");
    fs::write(&path, content)
        .map_err(|e| format!("Could not write _memory.md: {}", e))
}

/// List all `.md` filenames in the notes dir (excluding `_memory.md`).
pub fn list_notes(config: &BmoConfig) -> Result<Vec<String>, String> {
    let dir = notes_dir(config)?;
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Could not read notes dir: {}", e))?;

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && name != "_memory.md" {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    Ok(names)
}

/// Read a specific note by filename.
pub fn read_note(config: &BmoConfig, filename: &str) -> Result<String, String> {
    let dir = notes_dir(config)?;
    let path = dir.join(filename);
    // Prevent path traversal
    if path.parent() != Some(&dir) {
        return Err("Invalid filename".into());
    }
    fs::read_to_string(&path)
        .map_err(|e| format!("Could not read note '{}': {}", filename, e))
}

/// Write a note file.
pub fn write_note(config: &BmoConfig, filename: &str, content: &str) -> Result<(), String> {
    let dir = notes_dir(config)?;
    let path = dir.join(filename);
    // Prevent path traversal
    if path.parent() != Some(&dir) {
        return Err("Invalid filename".into());
    }
    fs::write(&path, content)
        .map_err(|e| format!("Could not write note '{}': {}", filename, e))
}
