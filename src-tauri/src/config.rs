use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Enums ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScreenSide {
    Left,
    #[default]
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotesMode {
    Obsidian,
    #[default]
    Local,
}

fn default_true() -> bool {
    true
}

// ── Config structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesConfig {
    pub mode: NotesMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obsidian_vault_path: Option<String>,
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self {
            mode: NotesMode::Local,
            obsidian_vault_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmoConfig {
    pub display_name: String,
    pub screen_side: ScreenSide,
    pub llm_provider: LlmProvider,
    #[serde(default)]
    pub llm_model: String,
    pub always_on_top: bool,
    pub launch_at_login: bool,
    #[serde(default = "default_true")]
    pub personality_enabled: bool,
    pub notes: NotesConfig,
}

impl Default for BmoConfig {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            screen_side: ScreenSide::Right,
            llm_provider: LlmProvider::None,
            llm_model: String::new(),
            always_on_top: false,
            launch_at_login: false,
            personality_enabled: true,
            notes: NotesConfig::default(),
        }
    }
}

impl BmoConfig {
    /// Returns `~/.bmo/config.toml`.
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".bmo").join("config.toml")
    }

    /// Load config from `~/.bmo/config.toml`.
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        toml::from_str(&contents)
            .map_err(|e| format!("Invalid config: {}", e))
    }

    /// Validate an API key by making a lightweight HTTP request.
    pub fn validate_api_key(provider: &LlmProvider, key: &str) -> Result<(), String> {
        let client = reqwest::blocking::Client::new();
        match provider {
            LlmProvider::OpenAI => {
                let resp = client
                    .get("https://api.openai.com/v1/models")
                    .header("Authorization", format!("Bearer {}", key))
                    .send()
                    .map_err(|e| format!("Network error: {}", e))?;
                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            LlmProvider::Anthropic => {
                let resp = client
                    .get("https://api.anthropic.com/v1/models")
                    .header("x-api-key", key)
                    .header("anthropic-version", "2023-06-01")
                    .send()
                    .map_err(|e| format!("Network error: {}", e))?;
                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            LlmProvider::None => Ok(()),
        }
    }

    /// Returns `~/.bmo/.env`.
    pub fn env_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".bmo").join(".env")
    }

    /// Env var name for a provider's API key.
    fn env_key_name(provider: &LlmProvider) -> Result<&'static str, String> {
        match provider {
            LlmProvider::Anthropic => Ok("ANTHROPIC_API_KEY"),
            LlmProvider::OpenAI => Ok("OPENAI_API_KEY"),
            LlmProvider::None => Err("No LLM provider configured.".into()),
        }
    }

    /// Save an API key for a specific provider to `~/.bmo/.env`.
    /// Preserves keys for other providers.
    pub fn save_api_key(provider: &LlmProvider, key: &str) -> Result<(), String> {
        let path = Self::env_path();
        let var_name = Self::env_key_name(provider)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create dir: {}", e))?;
        }

        // Read existing .env content (or start empty)
        let existing = fs::read_to_string(&path).unwrap_or_default();

        // Rebuild: keep lines for OTHER providers, replace/add this one
        let mut lines: Vec<String> = existing
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Keep comments, blank lines, and lines that aren't for this provider
                trimmed.is_empty()
                    || trimmed.starts_with('#')
                    || !trimmed.starts_with(var_name)
            })
            .map(|s| s.to_string())
            .collect();

        lines.push(format!("{}={}", var_name, key));

        let content = lines.join("\n") + "\n";
        fs::write(&path, &content)
            .map_err(|e| format!("Could not write .env: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("Could not set permissions: {}", e))?;
        }
        Ok(())
    }

    /// Load an API key for a specific provider from `~/.bmo/.env`.
    /// Falls back to migrating from old `~/.bmo/.credentials` if `.env` doesn't exist.
    pub fn load_api_key(provider: &LlmProvider) -> Result<String, String> {
        let var_name = Self::env_key_name(provider)?;
        let path = Self::env_path();

        // Migration: if old .credentials exists and .env doesn't, migrate
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let old_creds = PathBuf::from(&home).join(".bmo").join(".credentials");
        if !path.exists() && old_creds.exists() {
            if let Ok(old_key) = fs::read_to_string(&old_creds) {
                let old_key = old_key.trim().to_string();
                if !old_key.is_empty() {
                    // Best-effort: try to detect provider from key prefix
                    let migrate_provider = if old_key.starts_with("sk-ant-") {
                        LlmProvider::Anthropic
                    } else {
                        LlmProvider::OpenAI
                    };
                    let _ = Self::save_api_key(&migrate_provider, &old_key);
                    let _ = fs::remove_file(&old_creds);
                }
            }
        }

        let contents = fs::read_to_string(&path)
            .map_err(|_| format!("No API keys found. Run `bmo --settings` to add one."))?;

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((name, value)) = trimmed.split_once('=') {
                if name.trim() == var_name {
                    let key = value.trim().to_string();
                    if !key.is_empty() {
                        return Ok(key);
                    }
                }
            }
        }

        Err(format!(
            "No {} key found in ~/.bmo/.env. Run `bmo --settings` to add one.",
            match provider {
                LlmProvider::Anthropic => "Anthropic",
                LlmProvider::OpenAI => "OpenAI",
                LlmProvider::None => "LLM",
            }
        ))
    }

    /// Write config to `~/.bmo/config.toml`.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create dir: {}", e))?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&path, toml_str)
            .map_err(|e| format!("Could not write {}: {}", path.display(), e))
    }
}
