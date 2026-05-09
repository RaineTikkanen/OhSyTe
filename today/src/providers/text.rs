use crate::event::{Category, Event, MonthDay};
use crate::filter::EventFilter;
use crate::providers::EventProvider;
use chrono::{Datelike, Local, NaiveDate};
use log::error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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

#[derive(Debug)]
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

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let result = File::open(self.path.clone());
        let file = match result {
            Ok(f) => f,
            Err(e) => {
                error!("Error opening file {:?}: {}", self.path, e);
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
                    error!("Error reading line from file {:?}: {}", self.path, e);
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
                    let is_yearless = date_string.starts_with("--");
                    if is_yearless {
                        date_string = date_string.replace("--", "2000-");
                    }
                    let event: Event;
                    match NaiveDate::parse_from_str(&date_string, "%F") {
                        Ok(date) => {
                            let category = Category::from_str(&category_string);
                            if is_yearless {
                                event = Event::new_annual(
                                    MonthDay::new(date.month(), date.day()),
                                    description.clone(),
                                    category,
                                );
                            } else {
                                event = Event::new_singular(date, description.clone(), category);
                            }
                            if filter.accepts(&event) {
                                events.push(event);
                            }
                        }
                        Err(_) => {
                            error!("Invalid timestamp '{}'", date_string);
                        }
                    }
                    state = ReadingState::Date;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::filter::FilterBuilder;

    //Funktion tekemisessä hyödynnetty tekoälyä
    fn create_test_txt_file(path: &Path) {
        let content = "2024-01-01
Event 1
testing

2024-02-14
Event 2
testing

--02-14
Annual Event
Annual

";
        std::fs::write(path, content).unwrap();
    }

    fn get_test_events() -> [Event; 3] {
        return [
            Event::new_singular(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                String::from("Event 1"),
                Category::from_primary("testing"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
                String::from("Event 2"),
                Category::from_primary("testing"),
            ),
            Event::new_annual(
                MonthDay::new(2, 14),
                String::from("Annual Event"),
                Category::from_primary("Annual"),
            ),
        ];
    }

    #[test]
    fn reads_events_from_text_file() {
        let path = Path::new("test_events.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let mut events: Vec<Event> = Vec::new();
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        let test_events = get_test_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
        assert_eq!(events[2], test_events[2]);
        let _ = std::fs::remove_file(path);
    }
}
