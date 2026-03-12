use crate::config::{BmoConfig, LlmProvider, NotesConfig, NotesMode, ScreenSide};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveConfigPayload {
    pub display_name: String,
    pub screen_side: String,
    pub llm_provider: String,
    pub llm_model: String,
    pub always_on_top: bool,
    pub launch_at_login: bool,
    pub personality_enabled: bool,
    pub notes_mode: String,
    pub obsidian_vault_path: Option<String>,
}

#[tauri::command]
pub fn save_config(payload: SaveConfigPayload) -> Result<BmoConfig, String> {
    let config = BmoConfig {
        display_name: payload.display_name,
        screen_side: match payload.screen_side.as_str() {
            "left" => ScreenSide::Left,
            _ => ScreenSide::Right,
        },
        llm_provider: match payload.llm_provider.as_str() {
            "anthropic" => LlmProvider::Anthropic,
            "openai" => LlmProvider::OpenAI,
            _ => LlmProvider::None,
        },
        llm_model: payload.llm_model,
        always_on_top: payload.always_on_top,
        launch_at_login: payload.launch_at_login,
        personality_enabled: payload.personality_enabled,
        notes: NotesConfig {
            mode: match payload.notes_mode.as_str() {
                "obsidian" => NotesMode::Obsidian,
                _ => NotesMode::Local,
            },
            obsidian_vault_path: payload.obsidian_vault_path,
        },
    };
    config.save()?;
    Ok(config)
}

#[tauri::command]
pub fn get_masked_api_key(provider: String) -> Result<String, String> {
    let provider_enum = match provider.as_str() {
        "anthropic" => LlmProvider::Anthropic,
        "openai" => LlmProvider::OpenAI,
        _ => return Err("Unknown provider".into()),
    };
    let key = BmoConfig::load_api_key(&provider_enum)?;
    let len = key.len();
    if len > 10 {
        Ok(format!("{}...{}", &key[..6], &key[len - 4..]))
    } else {
        Ok("***".into())
    }
}

#[tauri::command]
pub fn save_api_key_cmd(provider: String, key: String) -> Result<(), String> {
    let provider_enum = match provider.as_str() {
        "anthropic" => LlmProvider::Anthropic,
        "openai" => LlmProvider::OpenAI,
        _ => return Err("Unknown provider".into()),
    };
    BmoConfig::save_api_key(&provider_enum, &key)
}
