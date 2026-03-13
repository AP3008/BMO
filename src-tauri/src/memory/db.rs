use crate::config::{BmoConfig, NotesMode};
use chrono::{Local, NaiveDate};
use std::fs;
use std::path::PathBuf;

const DATE_MARKER_PREFIX: &str = "<!-- date: ";
const DATE_MARKER_SUFFIX: &str = " -->";
const DEFAULT_MEMORY: &str = "# BMO Memory\n\nNo memories yet.\n";

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

/// Build a date marker line: `<!-- date: YYYY-MM-DD -->`
fn date_marker(date: NaiveDate) -> String {
    format!("{}{}{}", DATE_MARKER_PREFIX, date.format("%Y-%m-%d"), DATE_MARKER_SUFFIX)
}

/// Parse the date from the first line of memory content.
fn parse_memory_date(content: &str) -> Option<NaiveDate> {
    let first_line = content.lines().next()?;
    let trimmed = first_line.trim();
    if trimmed.starts_with(DATE_MARKER_PREFIX) && trimmed.ends_with(DATE_MARKER_SUFFIX) {
        let date_str = &trimmed[DATE_MARKER_PREFIX.len()..trimmed.len() - DATE_MARKER_SUFFIX.len()];
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
    } else {
        None
    }
}

/// Archive filename format: DD-MM-YYYY-memory.md
fn archive_filename(date: NaiveDate) -> String {
    format!("{}-memory.md", date.format("%d-%m-%Y"))
}

/// Read `_memory.md` from the notes dir with daily rotation.
/// - If missing: create with today's date + default content
/// - If date is before today: archive to DD-MM-YYYY-memory.md, reset with today's date
/// - If no date marker (legacy): prepend today's date, don't archive
/// - If same day: return as-is
pub fn get_memory(config: &BmoConfig) -> Option<String> {
    let dir = notes_dir(config).ok()?;
    let path = dir.join("_memory.md");
    let today = Local::now().date_naive();

    if !path.exists() {
        let content = format!("{}\n{}", date_marker(today), DEFAULT_MEMORY);
        let _ = fs::write(&path, &content);
        return Some(content);
    }

    let content = fs::read_to_string(&path).ok()?;

    match parse_memory_date(&content) {
        Some(file_date) if file_date < today => {
            // Archive yesterday's memory
            let archive_path = dir.join(archive_filename(file_date));
            let _ = fs::write(&archive_path, &content);
            // Reset with fresh content
            let fresh = format!("{}\n{}", date_marker(today), DEFAULT_MEMORY);
            let _ = fs::write(&path, &fresh);
            Some(fresh)
        }
        Some(_) => {
            // Same day — return as-is
            Some(content)
        }
        None => {
            // Legacy file without date marker — prepend today's date
            let updated = format!("{}\n{}", date_marker(today), content);
            let _ = fs::write(&path, &updated);
            Some(updated)
        }
    }
}

/// Write `_memory.md` to the notes dir, ensuring date marker is present.
pub fn save_memory(config: &BmoConfig, content: &str) -> Result<(), String> {
    let dir = notes_dir(config)?;
    let path = dir.join("_memory.md");

    let final_content = if content.trim_start().starts_with(DATE_MARKER_PREFIX) {
        content.to_string()
    } else {
        let today = Local::now().date_naive();
        format!("{}\n{}", date_marker(today), content)
    };

    fs::write(&path, final_content)
        .map_err(|e| format!("Could not write _memory.md: {}", e))
}

/// Read a memory archive by date string (DD-MM-YYYY).
pub fn read_memory_archive(config: &BmoConfig, date_str: &str) -> Result<String, String> {
    let dir = notes_dir(config)?;
    let filename = format!("{}-memory.md", date_str);
    let path = dir.join(&filename);
    if path.parent() != Some(&dir) {
        return Err("Invalid date string".into());
    }
    fs::read_to_string(&path)
        .map_err(|e| format!("Could not read memory archive '{}': {}", filename, e))
}

/// List all memory archive filenames (excluding `_memory.md`).
pub fn list_memory_archives(config: &BmoConfig) -> Result<Vec<String>, String> {
    let dir = notes_dir(config)?;
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Could not read notes dir: {}", e))?;

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with("-memory.md") && name != "_memory.md" {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    Ok(names)
}

/// List all `.md` filenames in the notes dir (excluding `_memory.md` and `*-memory.md` archives).
pub fn list_notes(config: &BmoConfig) -> Result<Vec<String>, String> {
    let dir = notes_dir(config)?;
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Could not read notes dir: {}", e))?;

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && name != "_memory.md" && !name.ends_with("-memory.md") {
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
