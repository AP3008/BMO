// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("init")   => bmo_lib::cli::init::run(),
        Some("config") => bmo_lib::cli::config::run(),
        Some("reset")  => bmo_lib::cli::reset::run(),
        Some("stop")   => bmo_lib::cli::stop::run(),
        Some("logs")   => bmo_lib::cli::logs::run(),
        _ => {
            if !bmo_lib::config_exists() { bmo_lib::cli::init::run(); }
            bmo_lib::run();
        }
    }
}
