use crate::config::BmoConfig;
use console::style;

pub fn run() {
    match BmoConfig::load() {
        Ok(config) => {
            println!();
            println!(
                "  {}",
                style("── BMO Config ──").cyan().bold()
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
            println!(
                "  {}",
                style(format!("Config file: {}", BmoConfig::config_path().display())).dim()
            );
            println!();
        }
        Err(_) => {
            eprintln!();
            eprintln!(
                "  {} {}",
                style("✘").red().bold(),
                style("No config found. Run `bmo init` first.").red()
            );
            eprintln!();
        }
    }
}
