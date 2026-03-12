pub mod cli;
pub mod commands;
pub mod config;
pub mod memory;
pub mod prompts;
pub mod scheduler;
pub mod google;
pub mod tools;

pub fn config_exists() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home).join(".bmo/config.toml").exists()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            use tauri::window::Color;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::switch_provider,
            commands::config::get_available_providers,
            commands::config::test_api_key,
            commands::config::get_models_for_provider,
            commands::config::switch_model,
            commands::config::save_config,
            commands::config::get_masked_api_key,
            commands::config::save_api_key_cmd,
            commands::llm::send_message,
            commands::llm::summarize_session,
            commands::notes::write_note,
            commands::notes::list_notes,
            commands::notes::read_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
