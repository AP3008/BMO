use crate::config::BmoConfig;
use crate::memory;

/// Sanitize a topic string for use as a filename.
fn sanitize_topic(topic: &str) -> String {
    topic
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[tauri::command]
pub fn write_note(topic: String, content: String) -> Result<String, String> {
    let config = BmoConfig::load()?;
    let now = chrono::Local::now();
    let date_prefix = now.format("%d-%m-%y").to_string();
    let safe_topic = sanitize_topic(&topic);
    let filename = format!("{}-{}.md", date_prefix, safe_topic);
    memory::db::write_note(&config, &filename, &content)?;
    Ok(filename)
}

#[tauri::command]
pub fn list_notes() -> Result<Vec<String>, String> {
    let config = BmoConfig::load()?;
    memory::db::list_notes(&config)
}

#[tauri::command]
pub fn read_note(filename: String) -> Result<String, String> {
    let config = BmoConfig::load()?;
    memory::db::read_note(&config, &filename)
}
