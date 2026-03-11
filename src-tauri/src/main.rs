// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bmo", about = "BMO — your AI desktop companion")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Open settings wizard to update current config
    #[arg(long)]
    settings: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the first-time setup wizard
    Init,
    /// Print current config
    Config,
    /// Reset BMO data
    Reset,
    /// View logs
    Logs,
    /// Stop BMO
    Stop,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => bmo_lib::cli::init::run(None),
        Some(Commands::Config) => bmo_lib::cli::config::run(),
        Some(Commands::Reset) => bmo_lib::cli::reset::run(),
        Some(Commands::Stop) => bmo_lib::cli::stop::run(),
        Some(Commands::Logs) => bmo_lib::cli::logs::run(),
        None if cli.settings => {
            let prefill = bmo_lib::config::BmoConfig::load().ok();
            bmo_lib::cli::init::run(prefill.as_ref());
        }
        None => {
            if !bmo_lib::config_exists() {
                bmo_lib::cli::init::run(None);
            }
            bmo_lib::run();
        }
    }
}
