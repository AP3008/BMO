use crate::config::{BmoConfig, LlmProvider};
use serde::Serialize;

// ── Model registry ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

pub fn models_for_provider(provider: &str) -> Vec<(&'static str, &'static str)> {
    match provider {
        "anthropic" => vec![
            ("claude-haiku-4-5-20251001", "Haiku 4.5"),
            ("claude-sonnet-4-5-20250929", "Sonnet 4.5"),
            ("claude-opus-4-5-20251101", "Opus 4.5"),
        ],
        "openai" => vec![
            ("gpt-4.1-nano", "GPT-4.1 nano"),
            ("gpt-4.1-mini", "GPT-4.1 mini"),
            ("gpt-5-mini", "GPT-5 mini"),
        ],
        _ => vec![],
    }
}

pub fn default_model_for_provider(provider: &str) -> &'static str {
    models_for_provider(provider)
        .first()
        .map(|(id, _)| *id)
        .unwrap_or("")
}

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
    // Reset model — old provider's model IDs aren't valid for the new one
    config.llm_model = String::new();
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

#[tauri::command]
pub fn get_models_for_provider(provider: String) -> Vec<ModelInfo> {
    models_for_provider(&provider)
        .into_iter()
        .map(|(id, label)| ModelInfo {
            id: id.into(),
            label: label.into(),
        })
        .collect()
}

#[tauri::command]
pub fn switch_model(model: String) -> Result<BmoConfig, String> {
    let mut config = BmoConfig::load()?;
    let provider_str = match config.llm_provider {
        LlmProvider::Anthropic => "anthropic",
        LlmProvider::OpenAI => "openai",
        LlmProvider::None => return Err("No provider configured".into()),
    };
    let valid = models_for_provider(provider_str)
        .iter()
        .any(|(id, _)| *id == model.as_str());
    if !valid {
        return Err(format!("Unknown model '{}' for provider '{}'", model, provider_str));
    }
    config.llm_model = model;
    config.save()?;
    Ok(config)
}
