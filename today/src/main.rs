use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, Subcommand};
use dirs;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;
use today::{Config, add_event, run};
mod birthday;
use birthday::handle_birthday;

use today::event::{Category, Event, MonthDay, Rule};

use today::filter::FilterBuilder;

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// List all event providers
    Providers,
    /// Adds an event to an event provider
    Add {
        #[arg(short, long, help = "Name of event provider")]
        provider_name: String,
        #[arg(
            short,
            long,
            help = "Date of event. Singular events: YYYY-MM-DD, Annual events: MM-DD, Rule-based events: 'third Monday in January'"
        )]
        date: String,
        #[arg(short = 'e', long, help = "Description of event")]
        description: String,
        #[arg(short, long, help = "Category of event. Format: primary[/secondary]")]
        category: String,
    },
}

#[derive(Parser, Debug)]
#[command(name = "today")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Command>,

    #[arg(short, long, help = "Event date in MM-DD format")]
    date: Option<String>,

    #[arg(
        short,
        long,
        help = "Categories to exclude, comma-separated a/b,c,d/*, \nExact match: [primary/secondary], Either primary or secondary: [category], Only primary: [primary/*]"
    )]
    exclude: Option<String>,

    #[arg(short, long, help = "No age calculation or birthday message")]
    no_birthday: bool,

    #[arg(
        short,
        long,
        help = "Event categories to search, comma-separated a/b,c,d/*, \nExact match: [primary/secondary], Either primary or secondary: [category], Only primary: [primary/*]"
    )]
    categories: Option<String>,

    #[arg(short, long, help = "Text to search from events")]
    text: Option<String>,
}

fn get_config_path(app_name: &str) -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join(app_name);

        if !config_path.exists() {
            error!("No config directory found at: {:?}", config_path);
            return None;
        } else {
            info!("Found config directory at: {:?}", config_path);
        }
        return Some(config_path);
    }
    None
}

fn handle_categories(categories_string: &str) -> Vec<Category> {
    categories_string
        .split(',')
        .map(|s| Category::from_str(s))
        .collect()
}

fn main() {
    env_logger::init();
    info!("Logger initialized");

    let args = Args::parse();
    debug!("args: {:?}", args);

    let month_day = if let Some(md) = args.date {
        debug!("month_day: {}", md);
        match MonthDay::from_str(&md) {
            Ok(d) => d,
            Err(e) => {
                error!("Error parsing month and day from '{}': {:?}", md, e);
                return;
            }
        }
    } else {
        let today: NaiveDate = Local::now().date_naive();
        match MonthDay::new(today.month(), today.day()) {
            Ok(md) => md,
            Err(e) => {
                error!("Error creating month day from today's date: {:?}", e);
                return;
            }
        }
    };

    debug!(
        "Using month_day: {:02}-{:02}",
        month_day.month(),
        month_day.day()
    );

    let exclude_categories: Option<Vec<Category>>;
    let categories: Option<Vec<Category>>;
    let description: Option<String>;

    if let Some(exclude_string) = args.exclude {
        debug!("exclude_string: {}", exclude_string);
        exclude_categories = Some(handle_categories(&exclude_string));
    } else {
        exclude_categories = None;
    }

    debug!("exclude_categories: {:#?}", exclude_categories);

    if let Some(categories_string) = args.categories {
        debug!("categories_string: {}", categories_string);
        categories = Some(handle_categories(&categories_string));
        debug!("categories: {:?}", categories);
    } else {
        categories = None;
    }

    if let Some(text) = args.text {
        debug!("text: {}", text);
        description = Some(text);
    } else {
        description = None;
    }

    let filter = FilterBuilder::new()
        .month_day(month_day)
        .exclude_categories(exclude_categories)
        .categories(categories)
        .text(description)
        .build();

    debug!("{:?}", filter);

    if args.no_birthday {
        info!("No birthday message will be shown");
    } else {
        handle_birthday();
    }

    const APP_NAME: &str = "today";
    match get_config_path(APP_NAME) {
        Some(path) => {
            let toml_path = path.join(format!("{}.toml", APP_NAME));
            info!("Looking for configuration file '{}'", &toml_path.display());

            if toml_path.exists() {
                info!("Found configuration file at '{}'", &toml_path.display());
            } else {
                error!(
                    "Configuration file not found at '{}'.",
                    &toml_path.display()
                );
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
            match args.cmd {
                Some(Command::Providers) => {
                    info!("Showing providers");
                    for provider in config.providers() {
                        print!("{}", provider.name());
                        match provider.kind().as_str() {
                            "text" | "csv" => println!(" *"),
                            "sqlite" => println!(" **"),
                            _ => println!(""),
                        }
                    }
                    println!("\n* = supports adding events");
                    println!("** = supports adding singular events only");
                }
                Some(Command::Add {
                    provider_name,
                    date,
                    description,
                    category,
                }) => {
                    let category = Category::from_str(&category);
                    let mut date_string = date;
                    let is_yearless = date_string.len() == 5;
                    let is_rule_based = !date_string.contains("-");
                    let event: Event;
                    if is_yearless {
                        date_string = format!("2000-{}", date_string)
                    }
                    if is_rule_based {
                        let rule = match Rule::parse(&date_string) {
                            Ok(r) => r,
                            Err(e) => {
                                error!(
                                    "Unable to parse rule-based date string '{}': {:?}",
                                    date_string, e
                                );
                                return;
                            }
                        };
                        event = Event::new_rule_based(rule, description, category);
                    } else {
                        let date = match chrono::NaiveDate::parse_from_str(&date_string, "%Y-%m-%d")
                        {
                            Ok(d) => d,
                            Err(_) => {
                                error!("Unable to parse date '{}'", date_string);
                                return;
                            }
                        };
                        if is_yearless {
                            event = Event::new_annual(
                                MonthDay::new(date.month(), date.day()).unwrap(),
                                description,
                                category,
                            );
                        } else {
                            event = Event::new_singular(date, description, category);
                        }
                    }
                    info!("Adding event '{}' to provider '{}'", event, provider_name);

                    add_event(&config, &path, &provider_name, &event);
                }
                _ => {
                    if let Err(e) = run(&config, &path, &filter) {
                        error!("Error: {}", e);
                        return;
                    }
                }
            }
        }
        None => {
            error! {"Can not configure the application"}
        }
    };
}
