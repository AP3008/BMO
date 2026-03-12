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
pub fn test_api_key() -> Result<String, String> {
    let config = BmoConfig::load()?;
    let key = BmoConfig::load_api_key(&config.llm_provider)?;
    let len = key.len();
    let masked = if len > 10 {
        format!("{}...{}", &key[..6], &key[len - 4..])
    } else {
        "***".to_string()
    };
    BmoConfig::validate_api_key(&config.llm_provider, &key)?;
    Ok(format!("Key {} validated successfully for {:?}", masked, config.llm_provider))
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
