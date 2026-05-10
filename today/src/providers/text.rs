use crate::event::{Category, Event, EventKind, MonthDay};
use crate::filter::EventFilter;
use crate::providers::{EventProvider, EventProviderError};
use chrono::{Datelike, NaiveDate};
use log::error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write, ErrorKind};
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

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        let file = match OpenOptions::new().append(true).open(self.path.clone()) {
            Ok(f) => f,
            Err(e) => {
                match e.kind(){
                    ErrorKind::PermissionDenied => error!("No write permission to file '{:?}.", self.path.clone()),
                    ErrorKind::NotFound => error!("File '{:?}' not found", self.path.clone()),
                    _=> error!("Unknown error while adding event to file '{:?}", self.path.clone()),
                };
                return Err(EventProviderError::OperationFailed)
            },
        };

        let mut writer = BufWriter::new(file);

        return match event.kind() {
            EventKind::Singular(date) => {
                _ = writeln!(
                    writer,
                    "{}\n{}\n{}\n",
                    date.to_string(),
                    event.description(),
                    event.category()
                );
                Ok(())
            }
            EventKind::Annual(month_day) => {
                _ = writeln!(
                    writer,
                    "--{}\n{}\n{}\n",
                    month_day.to_string(),
                    event.description(),
                    event.category()
                );
                Ok(())
            }
        };
    }

    fn add_is_supported(&self) -> bool {
        true
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
        let _ = std::fs::remove_file(&path);
        let test_events = get_test_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
        assert_eq!(events[2], test_events[2]);
    }

    #[test]
    fn reads_events_from_text_file_with_category_filter() {
        let path = Path::new("test_events_category_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let category = Category::from_primary("testing");
        let filter = FilterBuilder::new().category(category).build();
        let mut events: Vec<Event> = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].category(), Category::from_primary("testing"));
        assert_eq!(events[1].category(), Category::from_primary("testing"));
    }

    #[test]
    fn reads_events_from_text_file_with_text_filter() {
        let path = Path::new("test_events_text_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let filter = FilterBuilder::new().text("Event 1".to_string()).build();
        let mut events: Vec<Event> = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].description(), "Event 1");
    }

    #[test]
    fn reads_events_from_text_file_with_month_day_filter() {
        let path = Path::new("test_events_month_day_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let month_day = MonthDay::new(2, 14);
        let filter = FilterBuilder::new().month_day(month_day.clone()).build();
        let mut events: Vec<Event> = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].month_day(), month_day);
        assert_eq!(events[1].month_day(), month_day);
    }

    #[test]
    fn filter_denies_wrong_text() {
        let path = Path::new("test_events_wrong_text_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let filter = FilterBuilder::new().text("Wrong".to_string()).build();
        let mut events: Vec<Event> = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn adds_singular_event_to_text_file() {
        let path = Path::new("test_events_add_singular.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2024, 3, 5).unwrap(),
            String::from("Added Event"),
            Category::new("testing", "addition"),
        );
        _ = provider.add_event(&event);

        let mut events: Vec<Event> = Vec::new();
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 4);
        assert_eq!(events[3], event);
    }

    #[test]
    fn adds_annual_event_to_text_file() {
        let path = Path::new("test_events_add_annual.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let event = Event::new_annual(
            MonthDay::new(3, 5),
            String::from("Added Annual Event"),
            Category::new("testing", "addition"),
        );
        _ = provider.add_event(&event);

        let mut events: Vec<Event> = Vec::new();
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 4);
        assert_eq!(events[3], event);
    }
}
