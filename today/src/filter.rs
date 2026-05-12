use std::collections::HashSet;

use log::{debug, info};

use crate::{event::{Category, Event, MonthDay}, filter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FilterOption {
    MonthDay(MonthDay),
    Categories(Vec<Category>),
    Text(String),
    ExcludeCategories(Vec<Category>),
}

#[derive(Debug)]
pub struct EventFilter {
    options: HashSet<FilterOption>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            options: HashSet::new(),
        }
    }

    pub fn accepts_category(filter_category: &Category, event_category: &Category) -> bool {
        debug!("filter category: {:?}, event category: {:?}", filter_category, event_category);
        match filter_category.secondary() {
            Some(_) => {
                let result = filter_category==event_category;
                debug!("Exact match result: {}", result);
                result
            }
            None => {
                match event_category.secondary() {
                    Some(_) => {
                        let result = filter_category.primary()==event_category.primary() || filter_category.primary()==event_category.secondary().unwrap();
                        debug!("Primary match result: {}", result);
                        result
                    }
                    None => {
                        let result = filter_category.primary()==event_category.primary();
                        debug!("Primary match result: {}", result);
                        result
                    }
                }
            }
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
                FilterOption::Categories(categories) => categories.iter().any(|category| Self::accepts_category(category, &event.category())),
                FilterOption::Text(text) => event.description().contains(text),
                FilterOption::ExcludeCategories(categories) => !categories.iter().any(|category| Self::accepts_category(category, &event.category())),
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
    pub fn contains_categories(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Categories(_)))
    }

    pub fn contains_exclude_categories(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::ExcludeCategories(_)))
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
    pub fn categories(&self) -> Option<Vec<Category>> {
        for option in self.options.iter() {
            match option {
                FilterOption::Categories(categories) => return Some(categories.clone()),
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
    pub fn exclude_categories(&self) -> Option<Vec<Category>> {
        for option in self.options.iter() {
            match option {
                FilterOption::ExcludeCategories(categories) => return Some(categories.clone()),
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

    pub fn categories(mut self, categories: Vec<Category>) -> FilterBuilder {
        if !categories.is_empty() {
            self.options.insert(FilterOption::Categories(categories));
        }
        self
    }

    pub fn exclude_categories(mut self, categories: Vec<Category>) -> FilterBuilder {
        if !categories.is_empty() {
            self.options.insert(FilterOption::ExcludeCategories(categories));
        }
        self
    }

    pub fn text(mut self, text: String) -> FilterBuilder {
        if !text.is_empty() {
            self.options.insert(FilterOption::Text(text));
        }
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
            filter.contains_categories(),
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
    fn filter_accepts_right_exact_category() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category.clone(),
        );
        let filter = FilterBuilder::new().categories(vec![category]).build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_right_primary_category() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("programming", "rust"),
        );
        let filter_category = Category::from_primary("programming");
        let filter = FilterBuilder::new().categories(vec![filter_category]).build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_denies_wrong_primary_category() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("programming", "rust"),
        );
        let filter_category = Category::from_primary("wrong");
        let filter = FilterBuilder::new().categories(vec![filter_category]).build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_denies_wrong_secondary_category() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("programming", "rust"),
        );
        let filter_category = Category::new("programming", "wrong");
        let filter = FilterBuilder::new().categories(vec![filter_category]).build();

        assert!(!filter.accepts(&event));
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
        let filter = FilterBuilder::new().categories(vec![wrong_category]).build();

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
    fn filter_denies_wrong_description() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category,
        );
        let filter = FilterBuilder::new().text("Wrong".to_string()).build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_excludes_exact_category() {
        let category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            category.clone(),
        );
        let filter = FilterBuilder::new().exclude_categories(vec![category]).build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_excludes_primary_category() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("programming", "rust"),
        );
        let exclude_categories = vec![Category::from_primary("programming")];
        let filter = FilterBuilder::new().exclude_categories(exclude_categories).build();
        assert!(!filter.accepts(&event));
    }
}
