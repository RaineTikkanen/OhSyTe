use crate::event::{Category, Event, EventKind, MonthDay, Rule};
use crate::filter::EventFilter;
use crate::providers::{EventProvider, EventProviderError};
use chrono::{Datelike, NaiveDate, Local};
use csv::ReaderBuilder;
use log::{debug, error};
use std::fs::OpenOptions;
use std::io::{BufWriter, ErrorKind, Write};
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
    fn parse_event(
        category_string: &str,
        date_string: &str,
        description: &str,
    ) -> Option<Event> {
        let is_rule_based = !date_string.contains("-");
        let is_yearless = date_string.starts_with("--");

        let date_string = if is_yearless {
            let today = Local::now().date_naive();
            debug!("today: {}",today);
            debug!("today.leap_year: {}", today.leap_year());
            debug!("date_string: {}", date_string);
            if !today.leap_year() && date_string=="--02-29" {
                return None
            }
            let year_string = format!("{:04}-", today.year());
            date_string.replace("--", &year_string)
        } else {
            date_string.to_string()
        };
        if is_rule_based {
            debug!(
                "Parsing rule based event with date string '{}'",
                date_string
            );
            let rule = match Rule::parse(&date_string) {
                Ok(r) => r,
                Err(e) => {
                    error!("Error parsing rule '{}': {}", date_string, e);
                    return None;
                }
            };
            return Some(Event::new_rule_based(
                rule,
                description.to_string(),
                Category::from_str(&category_string),
            ));
        } else {
            match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(date) => {
                    let category = Category::from_str(&category_string);
                    if is_yearless {
                        return Some(Event::new_annual(
                            //Luotetaan chrono paketin validiointiin päivämäärän oikeellisuudesta, joten unwrap on turvallinen tässä
                            MonthDay::new(date.month(), date.day()).unwrap(),
                            description.to_string(),
                            category,
                        ));
                    } else {
                        return Some(Event::new_singular(date, description.to_string(), category));
                    }
                }
                Err(e) => {
                    error!("Error parsing date '{}': {}", date_string, e);
                    return None;
                }
            }
        }
    }
}


impl EventProvider for CSVFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let mut reader = match ReaderBuilder::new()
            .has_headers(false)
            .from_path(self.path.clone())
        {
            Ok(reader) => reader,
            Err(e) => {
                error!("Error reading CSV file '{}': {}", self.path.display(), e);
                return;
            }
        };
        for result in reader.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

            let date_string = record[0].to_string();
            let description = record[1].to_string();
            let category_string = record[2].to_string();
            let event = match Self::parse_event(&category_string, &date_string, &description) {
                Some(e) => e,
                None => {
                    continue;
                }
            };
            if filter.accepts(&event) {
                events.push(event)
            }
        }
    }

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        debug!("Adding event to provider '{}'", self.name);
        let file = match OpenOptions::new().append(true).open(self.path.clone()) {
            Ok(f) => f,
            Err(e) => {
                error! {"Error: {:?}", e};
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
                    "{},{},{}",
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
                    "--{},{},{}",
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
                    "{},{},{}",
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
    use crate::{filter::FilterBuilder};

    //Funktion tekemisessä hyödynnetty tekoälyä
    fn create_test_csv_file(path: &Path) {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path).unwrap();
        writeln!(file, "2024-01-01,Event 1,testing").unwrap();
        writeln!(file, "2024-02-14,Event 2,testing/test").unwrap();
        writeln!(file, "--02-14,Annual Event,annual/event").unwrap();
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
                Category::new("testing", "test"),
            ),
            Event::new_annual(
                MonthDay::new(2, 14).unwrap(),
                String::from("Annual Event"),
                Category::new("annual", "event"),
            ),
        ];
    }

    #[test]
    fn test_csv_provider() {
        let path = Path::new("test_csv.csv");
        create_test_csv_file(&path);
        let provider = CSVFileProvider::new("Test CSV Provider", &path);
        let filter = FilterBuilder::new().build();
        let mut events = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
        assert_eq!(events[2], test_events[2]);
    }

    #[test]
    fn test_csv_provider_with_primary_category_filter() {
        let path = Path::new("test_csv_2.csv");
        create_test_csv_file(&path);
        let provider = CSVFileProvider::new("Test CSV Provider", &path);
        let categories = Some(vec![Category::from_primary("testing")]);
        let filter = FilterBuilder::new().categories(categories).build();
        let mut events = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
    }

    #[test]
    fn test_csv_provider_with_full_category_filter() {
        let path = Path::new("test_csv_3.csv");
        create_test_csv_file(&path);
        let provider = CSVFileProvider::new("Test CSV Provider", &path);
        let categories = Some(vec![Category::new("annual", "event")]);
        let filter = FilterBuilder::new().categories(categories).build();
        let mut events = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_events[2]);
    }

    #[test]
    fn test_csv_provider_with_text_filter() {
        let path = Path::new("test_csv_4.csv");
        create_test_csv_file(&path);
        let provider = CSVFileProvider::new("Test CSV Provider", &path);
        let text = Some("Annual".to_string());
        let filter = FilterBuilder::new().text(text).build();
        let mut events = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_events[2]);
    }

    #[test]
    fn adds_singular_event_to_csv_file() {
        let path = Path::new("test_events_add_singular.csv");
        create_test_csv_file(path);
        let provider = CSVFileProvider::new("TestProvider", path);
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
    fn adds_annual_event_to_csv_file() {
        let path = Path::new("test_events_add_annual.csv");
        create_test_csv_file(path);
        let provider = CSVFileProvider::new("TestProvider", path);
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
        assert_eq!(events.len(), 4);
        assert_eq!(events[3], event);
    }

    #[test]
    fn adds_rule_based_event_to_csv_file() {
        let path = Path::new("test_events_add_rule_based.csv");
        create_test_csv_file(path);
        let provider = CSVFileProvider::new("TestProvider", path);
        let rule = Rule::parse("last Friday in March").unwrap();
        let event = Event::new_rule_based(
            rule,
            String::from("Added Rule-Based Event"),
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
