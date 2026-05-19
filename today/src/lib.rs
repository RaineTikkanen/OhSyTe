use std::error::Error;

pub mod event;
pub mod filter;
pub mod providers;

use event::Event;
use filter::EventFilter;
use log::{error, info};
use providers::{CSVFileProvider, EventProvider, SQLiteProvider, TextFileProvider, WebProvider};
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ProviderConfig {
    name: String,
    kind: String,
    resource: String,
}

impl ProviderConfig {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn kind(&self) -> String {
        self.kind.clone()
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    providers: Vec<ProviderConfig>,
}

impl Config {
    pub fn providers(&self) -> Vec<ProviderConfig> {
        self.providers.clone()
    }
}

fn create_providers(config: &Config, config_path: &Path) -> Vec<Box<dyn EventProvider>> {
    let mut providers: Vec<Box<dyn EventProvider>> = Vec::new();
    for cfg in config.providers.iter() {
        let path = config_path.join(&cfg.resource);
        match cfg.kind.as_str() {
            "text" => {
                let provider = TextFileProvider::new(&cfg.name, &path);
                providers.push(Box::new(provider))
            }
            "csv" => {
                let provider = CSVFileProvider::new(&cfg.name, &path);
                providers.push(Box::new(provider));
            }
            "sqlite" => {
                let provider = SQLiteProvider::new(&cfg.name, &path);
                providers.push(Box::new(provider));
            }
            "web" => {
                let provider = WebProvider::new(&cfg.name, &cfg.resource);
                providers.push(Box::new(provider));
            }
            _ => {
                error!("Provider kind not supported: {}", cfg.kind.as_str());
            }
        }
    }
    providers
}

pub fn run(
    config: &Config,
    config_path: &Path,
    filter: &EventFilter,
) -> Result<(), Box<dyn Error>> {
    let mut events: Vec<Event> = Vec::new();
    let providers = create_providers(config, config_path);
    let mut count = 0;
    for provider in providers {
        provider.get_events(&filter, &mut events);
        let new_count = events.len();
        info!(
            "Got {} events from provider '{}'",
            new_count - count,
            provider.name()
        );
        count = new_count;
    }
    for event in events {
        println!("{}", event);
    }
    Ok(())
}

pub fn add_event(config: &Config, config_path: &Path, provider_name: &str, event: &Event) {
    let providers = create_providers(config, config_path);

    let mut provider: Option<&dyn EventProvider> = None;
    for p in &providers {
        if p.name() == provider_name {
            provider = Some(p.as_ref());
            break;
        }
    }

    match provider {
        Some(p) => {
            if p.add_is_supported() {
                if let Err(_e) = p.add_event(event) {
                    error!("Unable to add event");
                }else{
                    println!("Event successfully added to provider '{}'", provider_name)
                }
            } else {
                error!("Adding events is not supported for provider '{}'", p.name());
            }
        }
        None => {
            error!("Unknown event provider '{}'", provider_name);
        }
    }
}
