use crate::event::{Category, Event};
use crate::providers::EventProvider;
use chrono::NaiveDate;
use sqlite::{Connection, State};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

//Pohjana käytetty kurssin repositoriota: https://github.com/jerekapyaho/tamk-ohsyte-2026/blob/main/08/today/src/providers/sqlite.rs 999aa88bf0e206156cf377f74199c7b686654af1

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
        let mut statement = match connection.prepare(category_query){
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

    fn get_events(&self, events: &mut Vec<Event>) {
        let connection = match Connection::open(self.path.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error connecting to database: {}", e);
                return;
            }
        };
        let category_map = self.get_categories(&connection);
        let event_query: String =
            "SELECT event_date, event_description, category_id FROM event".to_string();
        let mut statement = match connection.prepare(event_query){
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
    use crate::event::Event;
    use crate::providers::EventProvider;
    use crate::providers::SQLiteProvider;
    use chrono::NaiveDate;
    use sqlite::Connection;
    use std::path::Path;
    use std::fs;
    use crate::event::{Category};

    //Funktion tekemisessä hyödynnetty tekoälyn apua
    fn setup_test_db(path: &Path) {
        let connection = Connection::open(path).unwrap();
        
        connection.execute(
            "CREATE TABLE category (
                category_id INTEGER PRIMARY KEY,
                primary_name TEXT NOT NULL,
                secondary_name TEXT
            )"
        ).unwrap();
        
        connection.execute(
            "CREATE TABLE event (
                event_date TEXT NOT NULL,
                event_description TEXT NOT NULL,
                category_id INTEGER NOT NULL,
                FOREIGN KEY (category_id) REFERENCES category(category_id)
            )"
        ).unwrap();
        
        connection.execute(
            "INSERT INTO category (category_id, primary_name, secondary_name) VALUES 
             (1, 'history', 'politics'),
             (2, 'science', 'technology')"
        ).unwrap();
        
        connection.execute(
            "INSERT INTO event (event_date, event_description, category_id) VALUES 
             ('2023-01-15', 'Test historical event', 1),
             ('2023-02-20', 'Test tech event', 2)"
        ).unwrap();

        connection.execute(
            "INSERT INTO event (event_date, event_description, category_id) VALUES 
             ('2023-03-10', 'Test event with missing category', 999)"
        ).unwrap();

        connection.execute(
            "INSERT INTO event (event_date, event_description, category_id) VALUES 
             ('invalid-date', 'Test event with invalid date', 1)"
        ).unwrap();
    }

    #[test]
    fn successfull_read_events() {
        
        let path = Path::new("test_temp.db");
        
        setup_test_db(path);
        
        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        provider.get_events(&mut events);
        
        let _ =fs::remove_file(path);
        
        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            String::from("Test historical event"),
            Category::new("history", "politics"),
        );

        let test_event_second = Event::new_singular(
            NaiveDate::from_ymd_opt(2023, 2, 20).unwrap(),
            String::from("Test tech event"),
            Category::new("science", "technology"),
        );

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_event_first);
        assert_eq!(events[1], test_event_second);
    }

    #[test]
    fn missing_db_file() {
        let path = Path::new("non_existent.db");
        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        provider.get_events(&mut events);
        assert_eq!(events.len(), 0);
        let _ = fs::remove_file(path);
    }
}
