use crate::event::{Category, Event};
use crate::filter::EventFilter;
use crate::providers::EventProvider;
use chrono::NaiveDate;
use sqlite::{Connection, State};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn make_date_part(filter: &EventFilter) -> String {
    if let Some(month_day) = filter.month_day() {
        let md = format!("{:02}-{:02}", month_day.month(), month_day.day());
        format!("strftime('%m-%d', event_date) = '{}'", md)
    } else {
        "".to_string()
    }
}
fn make_category_part(filter: &EventFilter, category_map: &HashMap<i64, Category>) -> String {
    if let Some(filter_category) = filter.category() {
        let mut filter_category_id: Option<i64> = None;
        // Brute force search for maching category:
        for (category_id, category) in category_map {
            if *category == filter_category {
                filter_category_id = Some(*category_id);
                break;
            }
        }
        match filter_category_id {
            Some(id) => format!("category_id = {}", id),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    }
}

fn make_text_part(filter: &EventFilter) -> String {
    if let Some(text) = filter.text() {
        format!("event_description LIKE '%{}%'", text)
    } else {
        "".to_string()
    }
}

fn make_where_clause(filter: &EventFilter, category_map: &HashMap<i64, Category>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if filter.contains_month_day() {
        parts.push(make_date_part(filter));
    }
    if filter.contains_category() {
        parts.push(make_category_part(filter, category_map));
    }
    if filter.contains_text() {
        parts.push(make_text_part(filter));
    }
    let mut result = "".to_string();
    if !parts.is_empty() {
        result.push_str("WHERE ");
        result.push_str(&parts.join(" AND "));
    }
    result
}

pub struct SQLiteProvider {
    name: String,
    path: PathBuf,
}

impl SQLiteProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }

    fn get_categories(&self, connection: &Connection) -> HashMap<i64, Category> {
        let mut category_map: HashMap<i64, Category> = HashMap::new();
        let category_query = "SELECT category_id, primary_name, secondary_name FROM category";
        let mut statement = match connection.prepare(category_query) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error preparing category query: {}", e);
                return category_map;
            }
        };
        while let Ok(State::Row) = statement.next() {
            let category_id = statement.read::<i64, _>("category_id").unwrap();
            let primary = statement.read::<String, _>("primary_name").unwrap();
            let secondary = statement
                .read::<Option<String>, _>("secondary_name")
                .unwrap();
            let category = match secondary {
                Some(sec) => Category::new(&primary, &sec),
                None => Category::from_primary(&primary),
            };
            category_map.insert(category_id, category);
        }
        category_map
    }
}

impl EventProvider for SQLiteProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let connection = match Connection::open(self.path.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error connecting to database: {}", e);
                return;
            }
        };
        let category_map = self.get_categories(&connection);
        let where_clause = make_where_clause(filter, &category_map);
        let mut event_query: String = "SELECT event_date, event_description, category_id FROM
 event"
            .to_string();
        event_query.push(' '); // space between table name and WHERE clause
        event_query.push_str(&where_clause);
        let mut statement = match connection.prepare(event_query) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error preparing event query: {}", e);
                return;
            }
        };
        while let Ok(State::Row) = statement.next() {
            let date_string = match statement.read::<String, _>("event_date") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };
            let date = match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };
            let description = match statement.read::<String, _>("event_description") {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };
            let category_id = match statement.read::<i64, _>("category_id") {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };
            let category = match category_map.get(&category_id) {
                Some(c) => c,
                None => {
                    eprintln!("Error: could not find category matching the category id");
                    continue;
                }
            };
            events.push(Event::new_singular(
                date,
                description.to_string(),
                category.clone(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event::Category;
    use crate::event::Event;
    use crate::event::MonthDay;
    use crate::filter::FilterBuilder;
    use crate::providers::EventProvider;
    use crate::providers::SQLiteProvider;
    use chrono::NaiveDate;
    use sqlite::Connection;
    use std::fs;
    use std::path::Path;

    //Funktion tekemisessä hyödynnetty tekoälyn apua
    fn setup_test_db(path: &Path) {
        let connection = Connection::open(path).unwrap();

        connection
            .execute(
                "CREATE TABLE category (
                category_id INTEGER PRIMARY KEY,
                primary_name TEXT NOT NULL,
                secondary_name TEXT
            )",
            )
            .unwrap();

        connection
            .execute(
                "CREATE TABLE event (
                event_date TEXT NOT NULL,
                event_description TEXT NOT NULL,
                category_id INTEGER NOT NULL,
                FOREIGN KEY (category_id) REFERENCES category(category_id)
            )",
            )
            .unwrap();

        connection
            .execute(
                "INSERT INTO category (category_id, primary_name, secondary_name) VALUES 
             (1, 'history', 'politics'),
             (2, 'programming', 'technology'),
             (3, 'ice-hockey', 'sports' )",
            )
            .unwrap();

        connection
            .execute(
                "INSERT INTO event (event_date, event_description, category_id) VALUES 
             ('2023-01-15', 'historical event', 1),
             ('2024-01-15', 'tech event', 2),
             ('2025-03-25', 'historical sports event', 3)",
            )
            .unwrap();

    }

    fn create_faulty_lines_to_db(path: &Path){
        let connection = Connection::open(path).unwrap();
        connection
            .execute(
                "INSERT INTO event (event_date, event_description, category_id) VALUES 
             ('2023-03-10', 'Test event with missing category', 999),
             ('invalid-date', 'Test event with invalid date', 1)",
            )
            .unwrap();
    }

    #[test]
    fn successfull_read_events() {
        let path = Path::new("test_temp.db");

        setup_test_db(path);
        create_faulty_lines_to_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            String::from("historical event"),
            Category::new("history", "politics"),
        );

        let test_event_second = Event::new_singular(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            String::from("tech event"),
            Category::new("programming", "technology"),
        );
        let test_event_third = Event::new_singular(
            NaiveDate::from_ymd_opt(2025, 3, 25).unwrap(),
            String::from("historical sports event"),
            Category::new("ice-hockey", "sports"),
        );

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], test_event_first);
        assert_eq!(events[1], test_event_second);
        assert_eq!(events[2], test_event_third);

    }

    #[test]
    fn successfull_read_events_with_category_filter() {
        let path = Path::new("test_temp2.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let category = Category::new("history", "politics");
        let filter = FilterBuilder::new()
        .category(category)
        .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            String::from("historical event"),
            Category::new("history", "politics"),
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event_first);
    }

    #[test]
    fn successfull_read_events_with_date_filter() {
        let path = Path::new("test_temp3.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let month_day = MonthDay::new(1, 15);
        let filter = FilterBuilder::new()
        .month_day(month_day)
        .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            String::from("historical event"),
            Category::new("history", "politics"),
        );
        let test_event_second = Event::new_singular(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            String::from("tech event"),
            Category::new("programming", "technology"),
        );

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_event_first);
        assert_eq!(events[1], test_event_second);
    }

    #[test]
    fn successfull_read_events_with_text_filter() {
        let path = Path::new("test_temp4.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let filter = FilterBuilder::new()
        .text("hist".to_string())
        .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            String::from("historical event"),
            Category::new("history", "politics"),
        );

        let test_event_second = Event::new_singular(
            NaiveDate::from_ymd_opt(2025, 3, 25).unwrap(),
            String::from("historical sports event"),
            Category::new("ice-hockey", "sports"),
        );

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_event_first);
        assert_eq!(events[1], test_event_second);
    }
    
     #[test]
    fn successfull_read_events_with_text_and_date_filters() {
        let path = Path::new("test_temp5.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let month_day = MonthDay::new(1, 15);
        let filter = FilterBuilder::new()
        .text("hist".to_string())
        .month_day(month_day)
        .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            String::from("historical event"),
            Category::new("history", "politics"),
        );

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event_first);
    }
    

    #[test]
    fn missing_db_file() {
        let path = Path::new("non_existent.db");
        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        assert_eq!(events.len(), 0);
        let _ = fs::remove_file(path);
    }
}
