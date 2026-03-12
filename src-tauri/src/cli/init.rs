use crate::config::{BmoConfig, LlmProvider, NotesConfig, NotesMode, ScreenSide};
use console::{style, Term};
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Map a ScreenSide to its Select index.
fn screen_side_index(side: &ScreenSide) -> usize {
    match side {
        ScreenSide::Right => 0,
        ScreenSide::Left => 1,
    }
}

/// Map an LlmProvider to its Select index.
fn llm_provider_index(provider: &LlmProvider) -> usize {
    match provider {
        LlmProvider::OpenAI => 0,
        LlmProvider::Anthropic => 1,
        LlmProvider::None => 2,
    }
}

/// Map a NotesMode to its Select index.
fn notes_mode_index(mode: &NotesMode) -> usize {
    match mode {
        NotesMode::Local => 0,
        NotesMode::Obsidian => 1,
    }
}

/// Read input while displaying asterisks for each character.
fn read_masked_input(prompt: &str) -> String {
    let term = Term::stderr();
    let _ = term.write_str(&format!("{}: ", prompt));
    let mut key = String::new();
    loop {
        match term.read_key() {
            Ok(console::Key::Char(c)) => {
                key.push(c);
                let _ = term.write_str("*");
            }
            Ok(console::Key::Backspace) => {
                if !key.is_empty() {
                    key.pop();
                    let _ = term.clear_line();
                    let _ = term.write_str(&format!("\r{}: {}", prompt, "*".repeat(key.len())));
                }
            }
            Ok(console::Key::Enter) => {
                let _ = term.write_line("");
                break;
            }
            _ => {}
        }
    }
    key.trim().to_string()
}

/// Run the init/settings wizard. Pass `Some(&config)` to pre-fill with existing values.
pub fn run(prefill: Option<&BmoConfig>) {
    let is_settings = prefill.is_some();

    println!();
    if is_settings {
        println!(
            "  {}",
            style("╔══════════════════════════════════════╗")
                .cyan()
                .bold()
        );
        println!(
            "  {}",
            style("║   BMO Settings — update your config  ║")
                .cyan()
                .bold()
        );
        println!(
            "  {}",
            style("╚══════════════════════════════════════╝")
                .cyan()
                .bold()
        );
    } else {
        println!(
            "  {}",
            style("╔══════════════════════════════════╗")
                .cyan()
                .bold()
        );
        println!(
            "  {}",
            style("║   Welcome to BMO!  Let's set up. ║")
                .cyan()
                .bold()
        );
        println!(
            "  {}",
            style("╚══════════════════════════════════╝")
                .cyan()
                .bold()
        );
    }
    println!();
    println!(
        "  {}",
        style("Press Enter to keep the current value shown in brackets.")
            .dim()
    );
    println!();

    // ── 1. Display name ──────────────────────────────────────────────────────
    let display_name = if let Some(cfg) = prefill {
        Input::<String>::new()
            .with_prompt(style("What should BMO call you?").green().to_string())
            .default(cfg.display_name.clone())
            .interact_text()
            .unwrap_or_else(|_| std::process::exit(1))
    } else {
        Input::<String>::new()
            .with_prompt(style("What should BMO call you?").green().to_string())
            .interact_text()
            .unwrap_or_else(|_| std::process::exit(1))
    };

    // ── 2. Screen side ───────────────────────────────────────────────────────
    println!();
    let side_items = &["Right (default)", "Left"];
    let side_default = prefill.map_or(0, |c| screen_side_index(&c.screen_side));
    let side_idx = Select::new()
        .with_prompt(style("Which screen edge should BMO live on?").green().to_string())
        .items(side_items)
        .default(side_default)
        .interact()
        .unwrap_or(0);
    let screen_side = if side_idx == 1 {
        ScreenSide::Left
    } else {
        ScreenSide::Right
    };

    // ── 3. LLM provider ─────────────────────────────────────────────────────
    println!();
    println!(
        "  {}",
        style("── LLM Configuration ──").cyan().bold()
    );
    println!();
    let llm_items = &["OpenAI", "Anthropic", "Skip (no LLM)"];
    let llm_default = prefill.map_or(2, |c| llm_provider_index(&c.llm_provider));
    let llm_idx = Select::new()
        .with_prompt(style("Choose an LLM provider").green().to_string())
        .items(llm_items)
        .default(llm_default)
        .interact()
        .unwrap_or(2);
    let llm_provider = match llm_idx {
        0 => LlmProvider::OpenAI,
        1 => LlmProvider::Anthropic,
        _ => LlmProvider::None,
    };

    // ── 4. API key (OpenAI / Anthropic) ──────────────────────────────────────
    if matches!(llm_provider, LlmProvider::OpenAI | LlmProvider::Anthropic) {
        let (label, url) = match llm_provider {
            LlmProvider::OpenAI => ("OpenAI", "https://platform.openai.com/api-keys"),
            LlmProvider::Anthropic => {
                ("Anthropic", "https://console.anthropic.com/settings/keys")
            }
            _ => unreachable!(),
        };

        println!();
        println!(
            "  {} {}",
            style("Get your API key at:").dim(),
            style(url).cyan().underlined()
        );

        let open_browser = Confirm::new()
            .with_prompt(
                style(format!("Open the {} API key page in your browser?", label))
                    .green()
                    .to_string(),
            )
            .default(true)
            .interact()
            .unwrap_or(false);

        if open_browser {
            if let Err(e) = open::that(url) {
                eprintln!(
                    "  {}",
                    style(format!("Could not open browser: {}", e)).yellow()
                );
            }
        }

        println!();
        let key: String = read_masked_input(
            &style(format!("Paste your {} API key", label))
                .green()
                .to_string(),
        );

        if !key.is_empty() {
            // Validate
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::with_template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_strings(&[
                        "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " ",
                    ]),
            );
            spinner.set_message("Validating API key...");
            spinner.enable_steady_tick(Duration::from_millis(80));

            match BmoConfig::validate_api_key(&llm_provider, &key) {
                Ok(()) => {
                    spinner.finish_and_clear();
                    println!(
                        "  {} {}",
                        style("✔").green().bold(),
                        style("API key validated!").green().bold()
                    );
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    println!(
                        "  {} {}",
                        style("⚠").yellow(),
                        style(format!(
                            "Could not validate key ({}). Saved anyway — you can update it later.",
                            e
                        ))
                        .yellow()
                    );
                }
            }

            // Store to ~/.bmo/.env
            match BmoConfig::save_api_key(&llm_provider, &key) {
                Ok(()) => {
                    println!(
                        "  {} {}",
                        style("✔").green().bold(),
                        style("API key saved to ~/.bmo/.env").green()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "  {} {}",
                        style("⚠").yellow(),
                        style(format!("Could not save key: {}", e)).yellow()
                    );
                }
            }
        }
    }

    // ── 5. Notes storage ─────────────────────────────────────────────────────
    println!();
    println!(
        "  {}",
        style("── Notes Configuration ──").cyan().bold()
    );
    println!();
    let notes_items = &["Local (~/.bmo/notes/)", "Obsidian vault"];
    let notes_default = prefill.map_or(0, |c| notes_mode_index(&c.notes.mode));
    let notes_idx = Select::new()
        .with_prompt(style("Where should BMO store notes?").green().to_string())
        .items(notes_items)
        .default(notes_default)
        .interact()
        .unwrap_or(0);

    let mut obsidian_vault_path: Option<String> = None;
    let notes_mode = if notes_idx == 1 {
        let vault = if let Some(ref path) = prefill.and_then(|c| c.notes.obsidian_vault_path.clone()) {
            Input::<String>::new()
                .with_prompt(style("Path to your Obsidian vault").green().to_string())
                .default(path.clone())
                .validate_with(|input: &String| -> Result<(), &str> {
                    if PathBuf::from(input).is_dir() {
                        Ok(())
                    } else {
                        Err("Directory does not exist")
                    }
                })
                .interact_text()
                .unwrap_or_default()
        } else {
            Input::<String>::new()
                .with_prompt(style("Path to your Obsidian vault").green().to_string())
                .validate_with(|input: &String| -> Result<(), &str> {
                    if PathBuf::from(input).is_dir() {
                        Ok(())
                    } else {
                        Err("Directory does not exist")
                    }
                })
                .interact_text()
                .unwrap_or_default()
        };
        if !vault.is_empty() {
            obsidian_vault_path = Some(vault);
        }
        NotesMode::Obsidian
    } else {
        NotesMode::Local
    };

    // ── 6. Always on top ─────────────────────────────────────────────────────
    println!();
    println!(
        "  {}",
        style("── Preferences ──").cyan().bold()
    );
    println!();
    let aot_default = prefill.map_or(false, |c| c.always_on_top);
    let always_on_top = Confirm::new()
        .with_prompt(style("Keep BMO always on top?").green().to_string())
        .default(aot_default)
        .interact()
        .unwrap_or(false);

    // ── 7. Launch at login ───────────────────────────────────────────────────
    let lal_default = prefill.map_or(false, |c| c.launch_at_login);
    let launch_at_login = Confirm::new()
        .with_prompt(style("Launch BMO at login?").green().to_string())
        .default(lal_default)
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
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&[
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " ",
            ]),
    );
    spinner.set_message("Writing config...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let bmo_dir = PathBuf::from(&home).join(".bmo");

    fs::create_dir_all(bmo_dir.join("logs")).expect("Could not create ~/.bmo/logs/");

    if matches!(config.notes.mode, NotesMode::Local) {
        fs::create_dir_all(bmo_dir.join("notes")).expect("Could not create ~/.bmo/notes/");
    }

    config.save().expect("Could not write config");
    spinner.finish_and_clear();

    // ── Summary ──────────────────────────────────────────────────────────────
    println!();
    println!(
        "  {} {}",
        style("✔").green().bold(),
        style("Config saved to ~/.bmo/config.toml").green().bold()
    );
    println!();
    println!(
        "  {}  {}",
        style("Name:").white().bold(),
        style(&config.display_name).cyan()
    );
    println!(
        "  {}  {:?}",
        style("Screen side:").white().bold(),
        style(&config.screen_side).cyan()
    );
    println!(
        "  {}  {:?}",
        style("LLM provider:").white().bold(),
        style(&config.llm_provider).cyan()
    );
    println!(
        "  {}  {}",
        style("Always on top:").white().bold(),
        style(config.always_on_top).cyan()
    );
    println!(
        "  {}  {}",
        style("Launch at login:").white().bold(),
        style(config.launch_at_login).cyan()
    );
    println!(
        "  {}  {:?}",
        style("Notes mode:").white().bold(),
        style(&config.notes.mode).cyan()
    );
    if let Some(ref vault) = config.notes.obsidian_vault_path {
        println!(
            "  {}  {}",
            style("Obsidian vault:").white().bold(),
            style(vault).cyan()
        );
    }
    println!();
}
