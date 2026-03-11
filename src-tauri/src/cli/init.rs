use crate::config::{BmoConfig, LlmProvider, NotesConfig, NotesMode, ScreenSide};
use dialoguer::{Confirm, Input, Password, Select};
use std::fs;
use std::path::PathBuf;

pub fn run() {
    println!();
    println!("  ╔══════════════════════════════════╗");
    println!("  ║   Welcome to BMO!  Let's set up. ║");
    println!("  ╚══════════════════════════════════╝");
    println!();

    // 1 — Display name
    let display_name: String = Input::new()
        .with_prompt("What should BMO call you?")
        .interact_text()
        .unwrap_or_else(|_| std::process::exit(1));

    // 2 — Screen side
    let side_items = &["Right (default)", "Left"];
    let side_idx = Select::new()
        .with_prompt("Which screen edge should BMO live on?")
        .items(side_items)
        .default(0)
        .interact()
        .unwrap_or(0);
    let screen_side = if side_idx == 1 {
        ScreenSide::Left
    } else {
        ScreenSide::Right
    };

    // 3 — LLM provider
    let llm_items = &["OpenAI", "Anthropic", "Ollama", "Skip (no LLM)"];
    let llm_idx = Select::new()
        .with_prompt("Choose an LLM provider")
        .items(llm_items)
        .default(3)
        .interact()
        .unwrap_or(3);
    let llm_provider = match llm_idx {
        0 => LlmProvider::OpenAI,
        1 => LlmProvider::Anthropic,
        2 => LlmProvider::Ollama,
        _ => LlmProvider::None,
    };

    // 4 — API key (if provider chosen)
    if !matches!(llm_provider, LlmProvider::None | LlmProvider::Ollama) {
        let label = match llm_provider {
            LlmProvider::OpenAI => "OpenAI",
            LlmProvider::Anthropic => "Anthropic",
            _ => "API",
        };

        let key: String = Password::new()
            .with_prompt(format!("Paste your {} API key", label))
            .interact()
            .unwrap_or_default();

        if !key.is_empty() {
            match keyring::Entry::new("bmo", "api_key") {
                Ok(entry) => {
                    if let Err(e) = entry.set_password(&key) {
                        eprintln!("  Warning: could not save key to keychain: {}", e);
                        eprintln!("  You can set it later via your OS keychain.");
                    } else {
                        println!("  API key saved to OS keychain.");
                    }
                }
                Err(e) => {
                    eprintln!("  Warning: keychain not available: {}", e);
                }
            }
        }
    }

    // 5 — Notes storage
    let notes_items = &["Local (~/.bmo/notes/)", "Obsidian vault"];
    let notes_idx = Select::new()
        .with_prompt("Where should BMO store notes?")
        .items(notes_items)
        .default(0)
        .interact()
        .unwrap_or(0);

    let mut obsidian_vault_path: Option<String> = None;
    let notes_mode = if notes_idx == 1 {
        let vault: String = Input::new()
            .with_prompt("Path to your Obsidian vault")
            .validate_with(|input: &String| -> Result<(), &str> {
                if PathBuf::from(input).is_dir() {
                    Ok(())
                } else {
                    Err("Directory does not exist")
                }
            })
            .interact_text()
            .unwrap_or_default();

        if !vault.is_empty() {
            obsidian_vault_path = Some(vault);
        }
        NotesMode::Obsidian
    } else {
        NotesMode::Local
    };

    // 6 — Always on top
    let always_on_top = Confirm::new()
        .with_prompt("Keep BMO always on top?")
        .default(false)
        .interact()
        .unwrap_or(false);

    // 7 — Launch at login
    let launch_at_login = Confirm::new()
        .with_prompt("Launch BMO at login?")
        .default(false)
        .interact()
        .unwrap_or(false);

    // ── Build config ─────────────────────────────────────────────────────────
    let config = BmoConfig {
        display_name,
        screen_side,
        llm_provider,
        always_on_top,
        launch_at_login,
        notes: NotesConfig {
            mode: notes_mode,
            obsidian_vault_path,
        },
    };

    // ── Create directories ───────────────────────────────────────────────────
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let bmo_dir = PathBuf::from(&home).join(".bmo");

    fs::create_dir_all(bmo_dir.join("logs")).expect("Could not create ~/.bmo/logs/");

    if matches!(config.notes.mode, NotesMode::Local) {
        fs::create_dir_all(bmo_dir.join("notes")).expect("Could not create ~/.bmo/notes/");
    }

    // ── Write config ─────────────────────────────────────────────────────────
    config.save().expect("Could not write config");

    // ── Summary ──────────────────────────────────────────────────────────────
    println!();
    println!("  Done! Config saved to ~/.bmo/config.toml");
    println!();
    println!("  Name:           {}", config.display_name);
    println!("  Screen side:    {:?}", config.screen_side);
    println!("  LLM provider:   {:?}", config.llm_provider);
    println!("  Always on top:  {}", config.always_on_top);
    println!("  Launch at login: {}", config.launch_at_login);
    println!("  Notes mode:     {:?}", config.notes.mode);
    if let Some(ref vault) = config.notes.obsidian_vault_path {
        println!("  Obsidian vault: {}", vault);
    }
    println!();
}
