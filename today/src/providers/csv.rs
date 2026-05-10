use crate::event::{Category, Event, EventKind, MonthDay};
use crate::filter::EventFilter;
use crate::providers::{EventProvider, EventProviderError};
use chrono::{Datelike, NaiveDate};
use csv::ReaderBuilder;
use log::{debug, error};
use std::fs::{OpenOptions};
use std::io::{BufWriter, Write, ErrorKind};
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

            let mut date_string = record[0].to_string();
            let is_yearless = date_string.starts_with("--");

            if is_yearless {
                date_string = date_string.replace("--", "2000-");
            }
            
            let description = record[1].to_string();
            let category_string = record[2].to_string();

            match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(date) => {
                    let category = Category::from_str(&category_string);
                    let event: Event;
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
                    error!("Invalid date '{}'", date_string);
                }
            }
        }
    }

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        debug!("Adding event to provider '{}'", self.name);
        let file = match OpenOptions::new().append(true).open(self.path.clone()) {
            Ok(f) => f,
            Err(e) => {
                error!{"Error: {:?}", e};
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
                MonthDay::new(2, 14),
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
        let category = Category::from_primary("testing");
        let filter = FilterBuilder::new().category(category).build();
        let mut events = Vec::new();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_events[0]);
    }

    #[test]
    fn test_csv_provider_with_full_category_filter() {
        let path = Path::new("test_csv_3.csv");
        create_test_csv_file(&path);
        let provider = CSVFileProvider::new("Test CSV Provider", &path);
        let category = Category::new("annual","event");
        let filter = FilterBuilder::new().category(category).build();
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
        let filter = FilterBuilder::new().text("Annual".to_string()).build();
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
