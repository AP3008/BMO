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
    Ollama,
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
    pub always_on_top: bool,
    pub launch_at_login: bool,
    pub notes: NotesConfig,
}

impl Default for BmoConfig {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            screen_side: ScreenSide::Right,
            llm_provider: LlmProvider::None,
            always_on_top: false,
            launch_at_login: false,
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
