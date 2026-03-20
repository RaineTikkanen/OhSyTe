use dirs;
use std::fs;
use std::path::PathBuf;
use today::{Config, run};

fn get_config_path(app_name: &str) -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join(app_name);

        if !config_path.exists() {
            println!(
                "No config directory found, creating one at: {:?}",
                config_path
            );
            if let Err(_) = fs::create_dir(&config_path) {
                eprintln!("Unable to create config directory in {:?}", config_path);
                return None;
            }
        } else {
            println!("Found config directory at: {:?}", config_path);
        }
        return Some(config_path);
    }
    None
}

fn main() {
    const APP_NAME: &str = "today";
    if let Some(config_path) = get_config_path(APP_NAME) {
        let toml_path = config_path.join(format!("{}.toml", APP_NAME));
        println!("Looking for configuration file '{}'", &toml_path.display());

        if toml_path.exists() {
            println!("Found configuration file at '{}'", &toml_path.display());
        } else {
            eprintln!(
                "Configuration file not found at '{}'.",
                &toml_path.display()
            );
            println!("Creating empty configuration file.");
            if let Err(_) = fs::write(&toml_path, "") {
                eprintln!(
                    "Error creating empty configuration file at '{}'",
                    &toml_path.display()
                );
                return;
            }
        }

        let config_str = match fs::read_to_string(&toml_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Error reading configuration file from '{}': {}",
                    &toml_path.display(),
                    e
                );
                return;
            }
        };
        let config: Config = match toml::from_str(&config_str) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Error parsing configuration file from '{}': {}",
                    &toml_path.display(),
                    e
                );
                return;
            }
        };
        println!();
        if let Err(e) = run(&config, &config_path) {
            eprintln!("Error: {}", e);
            return;
        }
    }
}
