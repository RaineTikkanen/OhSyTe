use std::fmt;

use chrono::{Datelike, Local, NaiveDate, Weekday, Month};
use log::debug;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Clone)]
pub enum EventKind {
    Singular(NaiveDate),
    Annual(MonthDay),
    RuleBased(Rule),
}

#[derive(Debug, PartialEq, Clone)]
struct Rule {
 ordinal: Ordinal,
 weekday: Weekday,
 month: Month,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, EnumString)]
#[strum(ascii_case_insensitive)]
enum Ordinal {
 First = 1,
 Second = 2,
 Third = 3,
 Fourth = 4,
 Fifth = 5,
 Last = 6,
}

#[derive(Debug, PartialEq)]
pub struct Event {
    kind: EventKind,
    description: String,
    category: Category,
}

impl Event {
    pub fn category(&self) -> Category {
        self.category.clone()
    }
    pub fn description(&self) -> String {
        self.description.clone()
    }
    pub fn kind(&self) -> EventKind {
        self.kind.clone()
    }

    pub fn new_singular(date: NaiveDate, description: String, category: Category) -> Self {
        Self {
            kind: EventKind::Singular(date),
            description,
            category,
        }
    }

    pub fn new_annual(month_day: MonthDay, description: String, category: Category) -> Self {
        Self {
            kind: EventKind::Annual(month_day),
            description,
            category,
        }
    }

    pub fn year(&self) -> i32 {
        let today: NaiveDate = Local::now().date_naive();
        match &self.kind {
            EventKind::Singular(date) => date.year(),
            EventKind::Annual(_month_day) => today.year(),
            EventKind::RuleBased(_rule) => todo!("Rule-based events not implemented yet"),
        }
    }

    pub fn month_day(&self) -> MonthDay {
        match &self.kind {
            EventKind::Singular(date) => MonthDay {
                month: date.month(),
                day: date.day(),
            },
            EventKind::Annual(month_day) => MonthDay {
                month: month_day.month,
                day: month_day.day,
            },
            EventKind::RuleBased(_rule) => todo!("Rule-based events not implemented yet"),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.year(),
            self.description,
            self.category
        )
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct MonthDay {
    month: u32,
    day: u32,
}

#[derive(Debug)]
pub enum MonthDayParseError {
    InvalidFormat,
    InvalidMonth,
    InvalidDay,
}

impl MonthDay {
    pub fn new(month: u32, day: u32) -> Result<Self, MonthDayParseError> {
        let month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => match day {
                1..=31 => month,
                _ => return Err(MonthDayParseError::InvalidDay),
            },
            4 | 6 | 9 | 11 => match day {
                1..=30 => month,
                _ => return Err(MonthDayParseError::InvalidDay),
            },
            2 => match day {
                1..=29 => month,
                _ => return Err(MonthDayParseError::InvalidDay),
            },
            _ => return Err(MonthDayParseError::InvalidMonth),
        };
        Ok(Self { month, day })
    }

    pub fn from_str(s: &str) -> Result<Self, MonthDayParseError> {
        debug!("month_day string: {}", s);
        match s.len() {
            4 => {
                let month_string = &s[..2];
                let month = match month_string.parse() {
                    Ok(m) => m,
                    Err(_) => return Err(MonthDayParseError::InvalidMonth),
                };
                if month < 1 || month > 12 {
                    return Err(MonthDayParseError::InvalidMonth);
                }
                let day: u32 = match s[2..].parse() {
                    Ok(d) => d,
                    Err(_) => return Err(MonthDayParseError::InvalidDay),
                };
                if day < 1 || day > 31 {
                    return Err(MonthDayParseError::InvalidDay);
                }
                match Self::new(month, day) {
                    Ok(md) => Ok(md),
                    Err(e) => Err(e),
                }
            }
            _ => Err(MonthDayParseError::InvalidFormat),
        }
    }

    pub fn month(&self) -> u32 {
        self.month.clone()
    }

    pub fn day(&self) -> u32 {
        self.day.clone()
    }
}

impl fmt::Display for MonthDay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}-{:02}", self.month, self.day)
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Category {
    primary: String,
    secondary: Option<String>,
}

impl Category {
    pub fn new(primary: &str, secondary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: Some(secondary.to_string()),
        }
    }

    pub fn from_primary(primary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: None,
        }
    }

    pub fn from_str(s: &str) -> Category {
        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() < 2 {
            Category {
                primary: parts[0].to_string(),
                secondary: None,
            }
        } else {
            Category {
                primary: parts[0].to_string(),
                secondary: Some(parts[1].to_string()),
            }
        }
    }

    pub fn primary(&self) -> String {
        self.primary.clone()
    }

    pub fn secondary(&self) -> Option<String> {
        self.secondary.clone()
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.secondary {
            Some(sec) => write!(f, "{}/{}", self.primary, sec),
            None => write!(f, "{}", self.primary),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Month;

    use super::*;

    #[test]
    fn month_day_new() {
        let md = MonthDay::new(1, 1).unwrap();
        assert_eq!(md.month(), 1);
        assert_eq!(md.day(), 1);

        let md = MonthDay::new(12, 31).unwrap();
        assert_eq!(md.month(), 12);
        assert_eq!(md.day(), 31);

        let md = MonthDay::new(2, 29).unwrap();
        assert_eq!(md.month(), 2);
        assert_eq!(md.day(), 29);

        let md = MonthDay::new(4, 30).unwrap();
        assert_eq!(md.month(), 4);
        assert_eq!(md.day(), 30);
    }

    #[test]
    fn month_day_new_invalid() {
        MonthDay::new(0, 1).unwrap_err();
        MonthDay::new(1, 0).unwrap_err();
        MonthDay::new(13, 1).unwrap_err();
        MonthDay::new(1, 32).unwrap_err();
        MonthDay::new(2, 30).unwrap_err();
        MonthDay::new(3, 32).unwrap_err();
        MonthDay::new(4, 31).unwrap_err();
        MonthDay::new(5, 32).unwrap_err();
        MonthDay::new(6, 31).unwrap_err();
        MonthDay::new(7, 32).unwrap_err();
        MonthDay::new(8, 32).unwrap_err();
        MonthDay::new(9, 31).unwrap_err();
        MonthDay::new(10, 32).unwrap_err();
        MonthDay::new(11, 31).unwrap_err();
        MonthDay::new(12, 32).unwrap_err();
    }

    #[test]
    fn month_day_from_valid_str() {
        let md = MonthDay::from_str("0101").unwrap();
        assert_eq!(md.month(), 1);
        assert_eq!(md.day(), 1);

        let md = MonthDay::from_str("1231").unwrap();
        assert_eq!(md.month(), 12);
        assert_eq!(md.day(), 31);
    }

    #[test]
    fn month_day_from_invalid_string() {
        MonthDay::from_str("01-01").unwrap_err();
        MonthDay::from_str("1-1").unwrap_err();
        MonthDay::from_str("103").unwrap_err();
        MonthDay::from_str("1301").unwrap_err();
        MonthDay::from_str("1032").unwrap_err();
        MonthDay::from_str("0230").unwrap_err();
        MonthDay::from_str("0332").unwrap_err();
        MonthDay::from_str("ERRR").unwrap_err();
    }

    #[test]
    fn primary_category() {
        let category = Category::from_primary("testing");
        assert_eq!(category.primary(), "testing");
        assert_eq!(category.secondary(), None);
    }

    #[test]
    fn primary_secondary_category() {
        let category = Category::new("testing", "test");
        assert_eq!(category.primary(), "testing");
        assert_eq!(category.secondary(), Some("test".to_string()));
    }

    #[test]
    fn primary_category_from_str() {
        let category = Category::from_str("testing");
        assert_eq!(category.primary(), "testing");
        assert_eq!(category.secondary(), None);
    }

    #[test]
    fn primary_secondary_category_from_str() {
        let category = Category::from_str("testing/test");
        assert_eq!(category.primary(), "testing");
        assert_eq!(category.secondary(), Some("test".to_string()));
    }
}
