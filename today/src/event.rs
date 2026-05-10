use std::fmt;

use chrono::{Datelike, Local, NaiveDate};
use log::debug;

#[derive(Debug, PartialEq, Clone)]
pub enum EventKind {
    Singular(NaiveDate),
    Annual(MonthDay),
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
    pub fn new(month: u32, day: u32) -> Self {
        Self { month, day }
    }

    pub fn from_str(s: &str) -> Result<Self, MonthDayParseError> {
        debug!("month_day string: {}", s);
        match s.len() {
            4 => {
                let month_string = &s[..2];
                let month = month_string.parse().unwrap();
                if month < 1 || month > 12 {
                    return Err(MonthDayParseError::InvalidMonth);
                }
                let day: u32 = s[2..].parse().unwrap();
                if day < 1 || day > 31 {
                    return Err(MonthDayParseError::InvalidDay);
                }
                Ok(MonthDay { month, day })
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
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.secondary {
            Some(sec) => write!(f, "{}/{}", self.primary, sec),
            None => write!(f, "{}", self.primary),
        }
    }
}
