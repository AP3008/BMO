use crate::config::BmoConfig;

pub fn run() {
    let path = BmoConfig::config_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            println!("── {} ──", path.display());
            println!("{}", contents);
        }
        Err(_) => {
            eprintln!("No config found. Run `bmo init` first.");
        }
    }
}
