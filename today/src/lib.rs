use std::error::Error;

mod birthday;
mod event;
mod filter;
mod providers;

use birthday::handle_birthday;
use chrono::{Datelike, Local, NaiveDate};
use event::{Event, MonthDay};
use providers::{CSVFileProvider, EventProvider, SQLiteProvider, TextFileProvider, WebProvider};
use std::path::Path;

use serde::Deserialize;

use filter::FilterBuilder;

#[derive(Deserialize, Debug)]
pub struct ProviderConfig {
    name: String,
    kind: String,
    resource: String,
}
#[derive(Deserialize, Debug)]
pub struct Config {
    providers: Vec<ProviderConfig>,
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
                eprintln!("Unable to make provider: {:?}", cfg);
            }
        }
    }
    providers
}

pub fn run(config: &Config, config_path: &Path) -> Result<(), Box<dyn Error>> {
    handle_birthday();
    let mut events: Vec<Event> = Vec::new();
    let providers = create_providers(config, config_path);
    let mut count = 0;
    let filter = FilterBuilder::new().build();
    for provider in providers {
        provider.get_events(&filter, &mut events);
        let new_count = events.len();
        println!(
            "Got {} events from provider '{}'",
            new_count - count,
            provider.name()
        );
        count = new_count;
    }
    let today: NaiveDate = Local::now().date_naive();
    let today_month_day = MonthDay::new(today.month(), today.day());
    println!();
    for event in events {
        if today_month_day == event.month_day() {
            println!("{}", event);
        }
    }
    Ok(())
}
