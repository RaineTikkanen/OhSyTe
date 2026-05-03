use std::collections::HashSet;

use crate::event::{Category, Event, MonthDay};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FilterOption {
    MonthDay(MonthDay),
    Category(Category),
    Text(String),
}

pub struct EventFilter {
    options: HashSet<FilterOption>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            options: HashSet::new(),
        }
    }

    pub fn accepts(&self, event: &Event) -> bool {
        if self.options.is_empty() {
            return true;
        }

        let mut results: Vec<bool> = Vec::new();

        for option in self.options.iter() {
            let result = match option {
                FilterOption::MonthDay(month_day) => *month_day == event.month_day(),
                FilterOption::Category(category) => *category == event.category(),
                FilterOption::Text(text) => event.description().contains(text),
            };
            results.push(result);
        }

        results.iter().all(|&option| option)
    }

    pub fn contains_month_day(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::MonthDay(_)))
    }
    pub fn contains_category(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Category(_)))
    }

    pub fn contains_text(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Text(_)))
    }
    pub fn month_day(&self) -> Option<MonthDay> {
        for option in self.options.iter() {
            match option {
                FilterOption::MonthDay(month_day) => return Some(month_day.clone()),
                _ => (),
            }
        }
        None
    }
    pub fn category(&self) -> Option<Category> {
        for option in self.options.iter() {
            match option {
                FilterOption::Category(category) => return Some(category.clone()),
                _ => (),
            }
        }
        None
    }
    pub fn text(&self) -> Option<String> {
        for option in self.options.iter() {
            match option {
                FilterOption::Text(text) => return Some(text.clone()),
                _ => (),
            }
        }
        None
    }
}

pub struct FilterBuilder {
    options: HashSet<FilterOption>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            options: HashSet::new(),
        }
    }

    pub fn month_day(mut self, month_day: MonthDay) -> FilterBuilder {
        self.options.insert(FilterOption::MonthDay(month_day));
        self
    }

    pub fn category(mut self, category: Category) -> FilterBuilder {
        self.options.insert(FilterOption::Category(category));
        self
    }

    pub fn text(mut self, text: String) -> FilterBuilder {
        self.options.insert(FilterOption::Text(text));
        self
    }

    pub fn build(self) -> EventFilter {
        EventFilter {
            options: self.options,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Local, NaiveDate};

    #[test]
    fn creates_empty_filter() {
        let filter = FilterBuilder::new().build();
        let filter_content = [
            filter.contains_category(),
            filter.contains_month_day(),
            filter.contains_text(),
        ];
        assert_eq!(filter_content, [false, false, false]);
    }

    #[test]
    fn filter_accepts_anything() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category,
        );
        let filter = FilterBuilder::new().build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_right_date() {
        let category = Category::new("programming", "rust");
        let today = Local::now().date_naive();
        let month_day = MonthDay::new(today.month(), today.day());
        let event = Event::new_singular(today, "Test description".to_string(), category);
        let filter = FilterBuilder::new().month_day(month_day).build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_denies_wrong_date() {
        let category = Category::new("programming", "rust");
        let today = Local::now().date_naive();
        let month_day = MonthDay::new(today.month(), today.day());
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category,
        );
        let filter = FilterBuilder::new().month_day(month_day).build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_right_category() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category.clone(),
        );
        let filter = FilterBuilder::new().category(category).build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_denies_wrong_category() {
        let right_category = Category::new("programming", "rust");
        let wrong_category = Category::new("wrong", "category");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            right_category,
        );
        let filter = FilterBuilder::new().category(wrong_category).build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_right_description() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category,
        );
        let filter = FilterBuilder::new().text("Test".to_string()).build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_deniew_wrong_description() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category,
        );
        let filter = FilterBuilder::new().text("Wrong".to_string()).build();

        assert!(!filter.accepts(&event));
    }
}
