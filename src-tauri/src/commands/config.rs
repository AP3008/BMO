use crate::config::BmoConfig;

#[tauri::command]
pub fn get_config() -> Result<BmoConfig, String> {
    BmoConfig::load()
}
