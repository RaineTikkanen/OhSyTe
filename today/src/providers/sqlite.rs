use crate::event::{Category, Event};
use crate::filter::EventFilter;
use crate::providers::{EventProvider, EventProviderError};
use chrono::NaiveDate;
use log::{debug, error, info};
use sqlite::{Connection, State};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

    fn make_where_clause(
        &self,
        filter: &EventFilter,
        category_map: &HashMap<i64, Category>,
    ) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        debug!("Constructing where clause for filter: {:#?}", filter);
        if filter.contains_categories() {
            match self.make_categories_part(filter, category_map) {
                Some(part) => parts.push(part),
                None => {
                    info!(
                        "Filter has categories, but no matching category ids found in category map. Quitting."
                    );
                    return None;
                }
            }
        }
        debug!(
            "After processing categories, where clause parts: {:#?}",
            parts
        );
        if filter.contains_month_day() {
            match self.make_date_part(filter) {
                Some(part) => parts.push(part),
                None => debug!("No month and day part in filter"),
            }
        }
        debug!(
            "After processing month and day, where clause parts: {:#?}",
            parts
        );
        if filter.contains_exclude_categories() {
            match self.make_exclude_categories_part(filter, category_map) {
                Some(part) => parts.push(part),
                None => debug!("No exclude categories part in filter"),
            }
        }
        debug!(
            "After processing exclude categories, where clause parts: {:#?}",
            parts
        );
        if filter.contains_text() {
            match self.make_text_part(filter) {
                Some(part) => parts.push(part),
                None => debug!("No text part in filter"),
            }
        }
        debug!("After processing text, where clause parts: {:#?}", parts);

        let mut result = "".to_string();

        debug!("Constructing where clause from parts: {:#?}", parts);

        if !parts.is_empty() {
            result.push_str("WHERE ");
            result.push_str(&parts.join(" AND "));
        }
        debug!("Constructed where clause: '{}'", result);
        Some(result)
    }

    fn make_date_part(&self, filter: &EventFilter) -> Option<String> {
        if let Some(month_day) = filter.month_day() {
            let md = format!("{:02}-{:02}", month_day.month(), month_day.day());
            debug!("Constructed date part: '{}'", md);
            Some(format!("strftime('%m-%d', event_date) = '{}'", md))
        } else {
            None
        }
    }

    fn matching_categories_ids(
        &self,
        filter_categories: Vec<Category>,
        category_map: &HashMap<i64, Category>,
    ) -> Option<Vec<String>> {
        let mut category_ids: Vec<String> = Vec::new();
        for filter_category in filter_categories {
            for (category_id, category) in category_map {
                if EventFilter::categories_match(&filter_category, category) {
                    category_ids.push(category_id.to_string());
                    break;
                }
            }
        }
        if !category_ids.is_empty() {
            return Some(category_ids);
        } else {
            return None;
        }
    }

    fn make_categories_part(
        &self,
        filter: &EventFilter,
        category_map: &HashMap<i64, Category>,
    ) -> Option<String> {
        if let Some(filter_categories) = filter.categories() {
            debug!("Filter has categories: {:#?}", filter_categories);
            match self.matching_categories_ids(filter_categories, category_map) {
                Some(ids) => return Some(format!("category_id IN ({})", ids.join(", "))),
                None => return None,
            };
        } else {
            Some("".to_string())
        }
    }

    fn make_exclude_categories_part(
        &self,
        filter: &EventFilter,
        category_map: &HashMap<i64, Category>,
    ) -> Option<String> {
        if let Some(exclude_categories) = filter.exclude_categories() {
            match self.matching_categories_ids(exclude_categories, category_map) {
                Some(ids) => return Some(format!("category_id NOT IN ({})", ids.join(", "))),
                None => return None,
            };
        } else {
            None
        }
    }

    fn find_exact_category_id(
        category_map: &HashMap<i64, Category>,
        category: &Category,
    ) -> Option<i64> {
        for (category_id, cat) in category_map {
            if cat == category {
                return Some(*category_id);
            }
        }
        None
    }

    fn make_text_part(&self, filter: &EventFilter) -> Option<String> {
        if let Some(text) = filter.text() {
            debug!("Constructed text part: '{}'", text);
            Some(format!("event_description LIKE '%{}%'", text))
        } else {
            None
        }
    }

    fn get_categories(&self, connection: &Connection) -> HashMap<i64, Category> {
        let mut category_map: HashMap<i64, Category> = HashMap::new();
        let category_query = "SELECT category_id, primary_name, secondary_name FROM category";
        let mut statement = match connection.prepare(category_query) {
            Ok(s) => s,
            Err(e) => {
                error!("Error preparing category query: {}", e);
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
        debug!("Got category map: {:#?}", category_map);
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
                error!("Error connecting to database: {}", e);
                return;
            }
        };
        let category_map = self.get_categories(&connection);
        debug!("Category map: {:#?}", category_map);

        let where_clause = match self.make_where_clause(filter, &category_map) {
            Some(clause) => clause,
            None => return,
        };
        let mut event_query: String = "SELECT event_date, event_description, category_id FROM
 event"
            .to_string();
        event_query.push(' ');
        event_query.push_str(&where_clause);
        debug!("Constructed event query: '{}'", event_query);
        let mut statement = match connection.prepare(event_query) {
            Ok(s) => s,
            Err(e) => {
                error!("Error preparing event query: {}", e);
                return;
            }
        };
        while let Ok(State::Row) = statement.next() {
            let date_string = match statement.read::<String, _>("event_date") {
                Ok(s) => s,
                Err(e) => {
                    error!("Error: {}", e);
                    continue;
                }
            };
            let date = match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(d) => d,
                Err(e) => {
                    error!("Error: {}", e);
                    continue;
                }
            };
            let description = match statement.read::<String, _>("event_description") {
                Ok(d) => d,
                Err(e) => {
                    error!("Error: {}", e);
                    continue;
                }
            };
            let category_id = match statement.read::<i64, _>("category_id") {
                Ok(c) => c,
                Err(e) => {
                    error!("Error: {}", e);
                    continue;
                }
            };
            let category = match category_map.get(&category_id) {
                Some(c) => c,
                None => {
                    error!("Error: could not find category matching the category id");
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

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        let connection = match Connection::open(self.path.clone()) {
            Ok(c) => c,
            Err(e) => {
                error!("Error connecting to database: {}", e);
                return Err(EventProviderError::OperationFailed);
            }
        };
        let category_map = self.get_categories(&connection);
        let mut category_id: Option<i64> = None;
        Err(EventProviderError::OperationFailed)
    }

    fn add_is_supported(&self) -> bool {
        true
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

    fn create_faulty_lines_to_db(path: &Path) {
        let connection = Connection::open(path).unwrap();
        connection
            .execute(
                "INSERT INTO event (event_date, event_description, category_id) VALUES 
             ('2023-03-10', 'Test event with missing category', 999),
             ('invalid-date', 'Test event with invalid date', 1)",
            )
            .unwrap();
    }

    fn get_test_events() -> [Event; 3] {
        return [
            Event::new_singular(
                NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
                String::from("historical event"),
                Category::new("history", "politics"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                String::from("tech event"),
                Category::new("programming", "technology"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2025, 3, 25).unwrap(),
                String::from("historical sports event"),
                Category::new("ice-hockey", "sports"),
            ),
        ];
    }

    #[test]
    fn successful_read_events() {
        let path = Path::new("test_temp.db");

        setup_test_db(path);
        create_faulty_lines_to_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_events = get_test_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
        assert_eq!(events[2], test_events[2]);
    }

    #[test]
    fn successful_read_events_with_exact_categories_filter() {
        let path = Path::new("test_temp2.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let categories = Some(vec![
            Category::new("history", "politics"),
            Category::new("programming", "technology"),
        ]);
        let filter = FilterBuilder::new().categories(categories).build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);
        let test_events = get_test_events();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
    }

    #[test]
    fn successful_read_events_with_primary_category_filter() {
        let path = Path::new("test_temp7.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let category = Category::from_primary("history");
        let filter = FilterBuilder::new()
            .categories(Some(vec![category]))
            .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);
        let test_events = get_test_events();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_events[0]);
    }

    #[test]
    fn successful_read_events_with_exclude_category_filter() {
        let path = Path::new("test_temp6.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let exclude_categories = Some(vec![Category::new("programming", "technology")]);
        let filter = FilterBuilder::new()
            .exclude_categories(exclude_categories)
            .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);
        let test_events = get_test_events();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[2]);
    }

    #[test]
    fn successful_read_events_with_date_filter() {
        let path = Path::new("test_temp3.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let month_day = MonthDay::new(1, 15).unwrap();
        let filter = FilterBuilder::new().month_day(month_day).build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[1]);
    }

    #[test]
    fn successful_read_events_with_text_filter() {
        let path = Path::new("test_temp4.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let text = Some("hist".to_string());
        let filter = FilterBuilder::new().text(text).build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], test_events[0]);
        assert_eq!(events[1], test_events[2]);
    }

    #[test]
    fn successful_read_events_with_text_and_date_filters() {
        let path = Path::new("test_temp5.db");

        setup_test_db(path);

        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let month_day = MonthDay::new(1, 15).unwrap();
        let text = Some("hist".to_string());
        let filter = FilterBuilder::new()
            .text(text)
            .month_day(month_day)
            .build();
        provider.get_events(&filter, &mut events);

        let _ = fs::remove_file(path);

        let test_events = get_test_events();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_events[0]);
    }

    #[test]
    fn missing_db_file() {
        let path = Path::new("non_existent.db");
        let mut events: Vec<Event> = Vec::new();
        let provider = SQLiteProvider::new("test", path);
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        let _ = std::fs::remove_file(&path);
        assert_eq!(events.len(), 0);
    }
}
