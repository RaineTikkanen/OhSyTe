use crate::event::{Category, Event, EventKind, MonthDay, Rule};
use crate::filter::EventFilter;
use crate::providers::{EventProvider, EventProviderError};
use chrono::{Datelike, NaiveDate};
use log::{debug, error};
use serde::de;
use std::f32::consts::E;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};
use strum_macros::Display;

pub struct TextFileProvider {
    name: String,
    path: PathBuf,
}

#[derive(Debug)]
enum TextFileProviderError {
    ParseError,
}

impl TextFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }

    fn parse_event(
        category_string: &str,
        date_string: &str,
        description: &str,
    ) -> Result<Event, TextFileProviderError> {
        debug!("date_string: {}", date_string);
        debug!("category_string: {}", category_string);
        debug!("description: {}", description);

        let category = Category::from_str(&category_string);
        let is_yearless = date_string.starts_with("--");
        debug!("is_yearless: {}", is_yearless);

        let date_string = if is_yearless {
            date_string.replace("--", "2000-")
        } else {
            date_string.to_string()
        };
        if !date_string.contains("-") {
            debug!(
                "Parsing rule-based event with date string '{}'",
                date_string
            );
            let rule = match Rule::parse(&date_string) {
                Ok(r) => r,
                Err(e) => {
                    error!("Error parsing rule '{}': {}", date_string, e);
                    return Err(TextFileProviderError::ParseError);
                }
            };
            return Ok(Event::new_rule_based(
                rule,
                description.to_string(),
                category,
            ));
        } else {
            debug!(
                "Parsing date-based event with date string '{}'",
                date_string
            );
            match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(date) => {
                    if is_yearless {
                        return Ok(Event::new_annual(
                            //Luotetaan chrono paketin validiointiin päivämäärän oikeellisuudesta, joten unwrap on turvallinen tässä
                            MonthDay::new(date.month(), date.day()).unwrap(),
                            description.to_string(),
                            category,
                        ));
                    } else {
                        return Ok(Event::new_singular(date, description.to_string(), category));
                    }
                }
                Err(e) => {
                    error!("Error parsing date '{}': {}", date_string, e);
                    return Err(TextFileProviderError::ParseError);
                }
            }
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
                    let event =
                        match Self::parse_event(&category_string, &date_string, &description) {
                            Ok(e) => e,
                            Err(e) => {
                                error!("Error parsing event from file {:?}: {:?}", self.path, e);
                                continue;
                            }
                        };

                    if filter.accepts(&event) {
                        events.push(event);
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
                match e.kind() {
                    ErrorKind::PermissionDenied => {
                        error!("No write permission to file '{:?}.", self.path.clone())
                    }
                    ErrorKind::NotFound => error!("File '{:?}' not found", self.path.clone()),
                    _ => error!(
                        "Unknown error while adding event to file '{:?}",
                        self.path.clone()
                    ),
                };
                return Err(EventProviderError::OperationFailed);
            }
        };

        let mut writer = BufWriter::new(file);

        return match event.kind() {
            EventKind::Singular(date) => {
                let result = writeln!(
                    writer,
                    "{}\n{}\n{}\n",
                    date.to_string(),
                    event.description(),
                    event.category()
                );
                debug!("Write result: {:?}", result);
                Ok(())
            }
            EventKind::Annual(month_day) => {
                let result = writeln!(
                    writer,
                    "--{}\n{}\n{}\n",
                    month_day.to_string(),
                    event.description(),
                    event.category()
                );
                debug!("Write result: {:?}", result);
                Ok(())
            }
            EventKind::RuleBased(rule) => {
                let result = writeln!(
                    writer,
                    "{}\n{}\n{}\n",
                    rule.as_string(),
                    event.description(),
                    event.category()
                );
                debug!("Write result: {:?}", result);
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

    fn create_test_txt_file(path: &Path) {
        let content = "2024-01-01
Event 1
testing/test

2024-02-14
Event 2
testing

--02-14
Annual Event
Annual

first monday in january
Rule based event
rule-based/January

";
        std::fs::write(path, content).unwrap();
    }

    fn get_test_events() -> [Event; 4] {
        return [
            Event::new_singular(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                String::from("Event 1"),
                Category::new("testing", "test"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
                String::from("Event 2"),
                Category::from_primary("testing"),
            ),
            Event::new_annual(
                MonthDay::new(2, 14).unwrap(),
                String::from("Annual Event"),
                Category::from_primary("Annual"),
            ),
            Event::new_rule_based(
                Rule::parse("first monday in january").unwrap(),
                String::from("Rule based event"),
                Category::new("rule-based", "January"),
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
        assert_eq!(events.len(), 4);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
        assert_eq!(events[2], test_events[2]);
        assert_eq!(events[3], test_events[3]);
    }

    #[test]
    fn reads_events_from_text_file_with_primary_category_filter() {
        let path = Path::new("test_events_category_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let category = Category::from_primary("testing");
        let filter = FilterBuilder::new()
            .categories(Some(vec![category]))
            .build();
        let mut events: Vec<Event> = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].category(), Category::new("testing", "test"));
        assert_eq!(events[1].category(), Category::from_primary("testing"));
    }

    #[test]
    fn reads_events_from_text_file_with_exact_category_filter() {
        let path = Path::new("test_events_secondary_category_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let category = Category::new("testing", "test");
        let filter = FilterBuilder::new()
            .categories(Some(vec![category]))
            .build();
        let mut events: Vec<Event> = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].category(), Category::new("testing", "test"));
    }

    #[test]
    fn reads_events_from_text_file_with_text_filter() {
        let path = Path::new("test_events_text_filter.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let text = Some("Event 1".to_string());
        let filter = FilterBuilder::new().text(text).build();
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
        let month_day = MonthDay::new(2, 14).unwrap();
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
        let text = Some("Wrong".to_string());
        let filter = FilterBuilder::new().text(text).build();
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
        assert_eq!(events.len(), 5);
        assert_eq!(events[4], event);
    }

    #[test]
    fn adds_annual_event_to_text_file() {
        let path = Path::new("test_events_add_annual.txt");
        create_test_txt_file(path);
        let provider = TextFileProvider::new("TestProvider", path);
        let event = Event::new_annual(
            MonthDay::new(3, 5).unwrap(),
            String::from("Added Annual Event"),
            Category::new("testing", "addition"),
        );
        _ = provider.add_event(&event);

        let mut events: Vec<Event> = Vec::new();
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 5);
        assert_eq!(events[4], event);
    }

    #[test]
    fn parse_singular_event() {
        let category_string = "testing/test";
        let date_string = "2024-01-01";
        let description = "Event 1";
        let event_result = TextFileProvider::parse_event(category_string, date_string, description);
        assert!(event_result.is_ok());
        let event = event_result.unwrap();
        assert_eq!(event.description(), description);
        assert_eq!(event.category(), Category::new("testing", "test"));
        assert_eq!(
            event.kind(),
            EventKind::Singular(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        );
    }

    #[test]
    fn parse_annual_event() {
        let category_string = "testing/test";
        let date_string = "--02-14";
        let description = "Annual Event";
        let event_result = TextFileProvider::parse_event(category_string, date_string, description);
        assert!(event_result.is_ok());
        let event = event_result.unwrap();
        assert_eq!(event.description(), description);
        assert_eq!(event.category(), Category::new("testing", "test"));
        assert_eq!(
            event.kind(),
            EventKind::Annual(MonthDay::new(2, 14).unwrap())
        );
    }

    #[test]
    fn parse_rule_based_event() {
        let category_string = "rule-based/January";
        let date_string = "first monday in january";
        let description = "Rule based event";
        let event_result = TextFileProvider::parse_event(category_string, date_string, description);
        assert!(event_result.is_ok());
        let event = event_result.unwrap();
        assert_eq!(event.description(), description);
        assert_eq!(event.category(), Category::new("rule-based", "January"));
    }
}
