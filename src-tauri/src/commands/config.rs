use crate::config::{BmoConfig, LlmProvider};

#[tauri::command]
pub fn get_config() -> Result<BmoConfig, String> {
    BmoConfig::load()
}

#[tauri::command]
pub fn switch_provider(provider: String) -> Result<BmoConfig, String> {
    let mut config = BmoConfig::load()?;
    config.llm_provider = match provider.as_str() {
        "anthropic" => LlmProvider::Anthropic,
        "openai" => LlmProvider::OpenAI,
        _ => return Err("Unknown provider".into()),
    };
    // Verify the key exists for this provider
    BmoConfig::load_api_key(&config.llm_provider)?;
    config.save()?;
    Ok(config)
}

#[tauri::command]
pub fn get_available_providers() -> Result<Vec<String>, String> {
    let mut providers = vec![];
    if BmoConfig::load_api_key(&LlmProvider::Anthropic).is_ok() {
        providers.push("anthropic".to_string());
    }
    if BmoConfig::load_api_key(&LlmProvider::OpenAI).is_ok() {
        providers.push("openai".to_string());
    }
    Ok(providers)
}
