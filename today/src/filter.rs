use std::collections::HashSet;

use crate::event::{Category, Event, MonthDay};

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

    ///Checks if an event matches the filter. An event matches the filter if it matches all the filter options. If the filter has no options, it matches all events.
    pub fn accepts(&self, event: &Event) -> bool {
        if self.options.is_empty() {
            return true;
        }

        let mut results: Vec<bool> = Vec::new();

        for option in self.options.iter() {
            let result = match option {
                FilterOption::MonthDay(month_day) => *month_day == event.month_day(),
                FilterOption::Categories(categories) => categories
                    .iter()
                    .any(|category| categories_match(category, &event.category())),
                FilterOption::Text(text) => event
                    .description()
                    .to_lowercase()
                    .contains(text.to_lowercase().as_str()),
                FilterOption::ExcludeCategories(categories) => !categories
                    .iter()
                    .any(|category| categories_match(category, &event.category())),
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
            if let FilterOption::MonthDay(month_day) = option {
                return Some(month_day.clone());
            }
        }
        None
    }

    pub fn categories(&self) -> Option<Vec<Category>> {
        for option in self.options.iter() {
            if let FilterOption::Categories(categories) = option {
                return Some(categories.clone());
            }
        }
        None
    }

    pub fn text(&self) -> Option<String> {
        for option in self.options.iter() {
            if let FilterOption::Text(text) = option {
                return Some(text.clone());
            }
        }
        None
    }

    pub fn exclude_categories(&self) -> Option<Vec<Category>> {
        for option in self.options.iter() {
            if let FilterOption::ExcludeCategories(categories) = option {
                return Some(categories.clone());
            }
        }
        None
    }
}

/// Checks if a filter category matches an event category.
/// Category matches if:
/// - filter category has a secondary category of "*" and its primary category matches the event's primary category, or
/// - filter category matches the event category exactly, or
/// - filter category has no secondary category and its primary category matches either the event's primary category or the event's secondary category (if it exists).
pub fn categories_match(filter_category: &Category, event_category: &Category) -> bool {
    match filter_category.secondary() {
        Some(s) if s == "*" => filter_category.primary() == event_category.primary(),
        Some(_) => filter_category == event_category,
        None => match event_category.secondary() {
            Some(event_secondary) => {
                filter_category.primary() == event_category.primary()
                    || filter_category.primary() == event_secondary
            },
            None => filter_category.primary() == event_category.primary(),
        },
    }
}

pub struct FilterBuilder {
    options: HashSet<FilterOption>,
}

impl FilterBuilder {
    ///Builder for creating an EventFilter. Allows chaining methods to set filter options and then build the filter.
    /// If no options are set, builder will create a filter that accepts all events.
    ///
    /// # Examples:
    /// ### Filter with all options set:
    /// ```
    /// # use today::{
    /// #    event::{Category, Event, MonthDay},
    /// # };
    /// # use today::filter::FilterBuilder;
    /// #
    /// let filter = FilterBuilder::new()
    ///     .month_day(MonthDay::new(3, 5).unwrap())
    ///     .categories(Some(vec![Category::from_primary("programming")]))
    ///     .exclude_categories(Some(vec![Category::from_primary("javascript")]))
    ///     .text(Some("test".to_string()))
    ///     .build();
    /// ```
    /// ### Empty filter that accepts all events:
    ///
    /// ```
    /// # use today::filter::FilterBuilder;
    /// let filter = FilterBuilder::new().build();
    /// ```
    pub fn new() -> Self {
        Self {
            options: HashSet::new(),
        }
    }

    ///Sets the month and day to accept
    pub fn month_day(mut self, month_day: MonthDay) -> FilterBuilder {
        self.options.insert(FilterOption::MonthDay(month_day));
        self
    }

    ///Sets the categories to accept
    pub fn categories(mut self, categories: Option<Vec<Category>>) -> FilterBuilder {
        if let Some(c) = categories {
            self.options.insert(FilterOption::Categories(c));
        }
        self
    }

    ///Sets the categories to not accept
    pub fn exclude_categories(
        mut self,
        exclude_categories: Option<Vec<Category>>,
    ) -> FilterBuilder {
        if let Some(e) = exclude_categories {
            self.options.insert(FilterOption::ExcludeCategories(e));
        }
        self
    }

    ///Sets the text to search for in the event description
    pub fn text(mut self, text: Option<String>) -> FilterBuilder {
        if let Some(t) = text {
            self.options.insert(FilterOption::Text(t));
        }
        self
    }

    ///Creates the EventFilter with the specified options
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
    fn creates_filter_with_month_day() {
        let month_day = MonthDay::new(3, 5).unwrap();
        let filter = FilterBuilder::new().month_day(month_day.clone()).build();
        assert_eq!(filter.month_day(), Some(month_day));
    }

    #[test]
    fn creates_filter_with_categories() {
        let categories = vec![Category::new("programming", "rust")];
        let filter = FilterBuilder::new()
            .categories(Some(categories.clone()))
            .build();
        assert_eq!(filter.categories(), Some(categories));
    }

    #[test]
    fn creates_filter_with_text() {
        let text = "test".to_string();
        let filter = FilterBuilder::new().text(Some(text.clone())).build();
        assert_eq!(filter.text(), Some(text));
    }

    #[test]
    fn creates_filter_with_exclude_categories() {
        let exclude_categories = vec![
            Category::new("programming", "rust"),
            Category::new("holiday", "christmas"),
        ];
        let filter = FilterBuilder::new()
            .exclude_categories(Some(exclude_categories.clone()))
            .build();
        assert_eq!(filter.exclude_categories(), Some(exclude_categories));
    }

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
        let month_day = MonthDay::new(today.month(), today.day()).unwrap();
        let event = Event::new_singular(today, "Test description".to_string(), category);
        let filter = FilterBuilder::new().month_day(month_day).build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_denies_wrong_date() {
        let category = Category::new("programming", "rust");
        let today = Local::now().date_naive();
        let month_day = MonthDay::new(today.month(), today.day()).unwrap();
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
        let filter = FilterBuilder::new()
            .categories(Some(vec![category]))
            .build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_primary_only_filter_matching_event_primary_category() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("programming", "rust"),
        );
        let filter_category = Category::from_primary("programming");
        let filter = FilterBuilder::new()
            .categories(Some(vec![filter_category]))
            .build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_primary_only_filter_matching_event_secondary_category() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2000, 1, 2).unwrap(),
            "Another test".to_string(),
            Category::new("rust", "programming"),
        );
        let filter_category = Category::from_primary("programming");
        let filter = FilterBuilder::new()
            .categories(Some(vec![filter_category]))
            .build();

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
        let filter = FilterBuilder::new()
            .categories(Some(vec![filter_category]))
            .build();

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
        let filter = FilterBuilder::new()
            .categories(Some(vec![filter_category]))
            .build();

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
        let filter = FilterBuilder::new()
            .categories(Some(vec![wrong_category]))
            .build();

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
        let text = Some("Test".to_string());
        let filter = FilterBuilder::new().text(text).build();

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
        let text = Some("Wrong".to_string());
        let filter = FilterBuilder::new().text(text).build();

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
        let filter = FilterBuilder::new()
            .exclude_categories(Some(vec![category]))
            .build();
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
        let filter = FilterBuilder::new()
            .exclude_categories(Some(exclude_categories))
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn category_is_not_case_sensitive() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("programming", "rust"),
        );
        let filter_category = Category::new("ProgRamMing", "rUsT");
        let filter = FilterBuilder::new()
            .categories(Some(vec![filter_category]))
            .build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn exclude_category_is_not_case_sensitive() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Test description".to_string(),
            Category::new("prOgrAmminG", "RuSt"),
        );
        let exclude_category = Category::new("ProgRamMing", "rUsT");
        let filter = FilterBuilder::new()
            .exclude_categories(Some(vec![exclude_category]))
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn text_search_is_not_case_sensitive() {
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "TesT dEsCripTioN".to_string(),
            Category::new("programming", "rust"),
        );
        let text = Some("tESt".to_string());
        let filter = FilterBuilder::new().text(text).build();
        assert!(filter.accepts(&event));

        let text = Some("DeScrIPtiOn".to_string());
        let filter = FilterBuilder::new().text(text).build();
        assert!(filter.accepts(&event));
    }
}
