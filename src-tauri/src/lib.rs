pub mod cli;
pub mod commands;
pub mod memory;
pub mod scheduler;
pub mod google;

pub fn config_exists() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home).join(".bmo/config.toml").exists()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
