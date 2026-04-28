use crate::event::{Category, Event};
use crate::providers::EventProvider;

use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::path::{Path, PathBuf};

pub struct CSVFileProvider {
    name: String,
    path: PathBuf,
}

impl CSVFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }
}

impl EventProvider for CSVFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, events: &mut Vec<Event>) {
        let mut reader = match ReaderBuilder::new()
            .has_headers(false)
            .from_path(self.path.clone())
        {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error reading CSV file '{}': {}", self.path.display(), e);
                return;
            }
        };
        for result in reader.records() {
            let record = result.unwrap();
            let date_string = record[0].to_string();
            let description = record[1].to_string();
            let category_string = record[2].to_string();

            match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(date) => {
                    let category = Category::from_str(&category_string);
                    let event = Event::new_singular(date, description.clone(), category);
                    events.push(event);
                }
                Err(_) => {
                    eprintln!("Invalid date '{}'", date_string);
                }
            }
        }
    }
}
