use rusqlite::{Connection, params};
use std::path::PathBuf;

/// Returns `~/.bmo/bmo.db`.
fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".bmo").join("bmo.db")
}

/// Open (or create) the database and run migrations.
pub fn init_db() -> Result<Connection, String> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create ~/.bmo/: {}", e))?;
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memory_summary (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            summary TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );"
    ).map_err(|e| format!("Migration error: {}", e))?;

    Ok(conn)
}

/// Get the current rolling summary, if one exists.
pub fn get_summary(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT summary FROM memory_summary WHERE id = 1",
        [],
        |row| row.get(0),
    ).ok()
}

/// Upsert the rolling summary.
pub fn save_summary(conn: &Connection, summary: &str) -> Result<(), String> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO memory_summary (id, summary, updated_at) VALUES (1, ?1, ?2)",
        params![summary, now],
    ).map_err(|e| format!("Could not save summary: {}", e))?;
    Ok(())
}
