use crate::event::{Category, Event};
use crate::providers::EventProvider;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use chrono::NaiveDate;

pub struct TextFileProvider {
    name: String,
    path: PathBuf,
}

impl TextFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }
}

enum ReadingState {
    Date,
    Description,
    Category,
    Separator,
}

impl EventProvider for TextFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, events: &mut Vec<Event>) {
        let result = File::open(self.path.clone());
        let file = match result {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error opening file {:?}: {}", self.path, e);
                return;
            }
        };
        let reader = BufReader::new(file);

        let mut state = ReadingState::Date;
        let mut date_string = String::new();
        let mut description = String::new();
        let mut category_string = String::new();
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading line from file {:?}: {}", self.path, e);
                    continue;
                }
            };

            match state {
                ReadingState::Date => {
                    date_string = line;
                    state = ReadingState::Description;
                }
                ReadingState::Description => {
                    description = line;
                    state = ReadingState::Category;
                }
                ReadingState::Category => {
                    category_string = line;
                    state = ReadingState::Separator;
                }
                ReadingState::Separator => {
                    match NaiveDate::parse_from_str(&date_string, "%F") {
                        Ok(date) => {
                            let category = Category::from_str(&category_string);
                            let event = Event::new_singular(date, description.clone(), category);
                            events.push(event);
                        }
                        Err(_) => {
                            eprintln!("Invalid timestamp '{}'", date_string);
                        }
                    }
                    state = ReadingState::Date;
                }
            }
        }
    }
}
