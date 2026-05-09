use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use dirs;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;
use today::{Config, run};


use today::event::MonthDay;

use today::filter::{EventFilter, FilterBuilder};

#[derive(Parser, Debug)]
#[command(name = "today")]
struct Args {
    #[arg(short, long, help = "Event date in MMDD format")]
    date: Option<String>,
}

fn get_config_path(app_name: &str) -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join(app_name);

        if !config_path.exists() {
            info!(
                "No config directory found, creating one at: {:?}",
                config_path
            );
            if let Err(_) = fs::create_dir(&config_path) {
                info!("Unable to create config directory in {:?}", config_path);
                return None;
            }
        } else {
            info!("Found config directory at: {:?}", config_path);
        }
        return Some(config_path);
    }
    None
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    println!("args: {:?}", args);

    let month_day = if let Some(md) = args.date {
        debug!("month_day: {}", md);
        MonthDay::from_str(&md)
    } else {
        let today: NaiveDate = Local::now().date_naive();
        MonthDay::new(today.month(), today.day())
    };

    let filter: EventFilter = FilterBuilder::new().month_day(month_day).build();

    const APP_NAME: &str = "today";
    if let Some(config_path) = get_config_path(APP_NAME) {
        let toml_path = config_path.join(format!("{}.toml", APP_NAME));
        info!("Looking for configuration file '{}'", &toml_path.display());

        if toml_path.exists() {
            info!("Found configuration file at '{}'", &toml_path.display());
        } else {
            error!(
                "Configuration file not found at '{}'.",
                &toml_path.display()
            );
            info!("Creating empty configuration file.");
            if let Err(_) = fs::write(&toml_path, "") {
                error!(
                    "Error creating empty configuration file at '{}'",
                    &toml_path.display()
                );
                return;
            }
        }

        let config_str = match fs::read_to_string(&toml_path) {
            Ok(s) => s,
            Err(e) => {
                error!(
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
                error!(
                    "Error parsing configuration file from '{}': {}",
                    &toml_path.display(),
                    e
                );
                return;
            }
        };
        if let Err(e) = run(&config, &config_path, &filter) {
            error!("Error: {}", e);
            return;
        }
    }
}
