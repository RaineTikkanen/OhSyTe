use std::fmt;

use chrono::{Datelike, Local, Month, NaiveDate, Weekday as ChronoWeekday};
use log::debug;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Clone)]
pub enum EventKind {
    Singular(NaiveDate),
    Annual(MonthDay),
    RuleBased(Rule),
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

    pub fn new_rule_based(rule: Rule, description: String, category: Category) -> Self {
        Self {
            kind: EventKind::RuleBased(rule),
            description,
            category,
        }
    }

    pub fn year(&self) -> i32 {
        let today: NaiveDate = Local::now().date_naive();
        match &self.kind {
            EventKind::Singular(date) => date.year(),
            EventKind::Annual(_month_day) => today.year(),
            EventKind::RuleBased(_rule) => today.year(),
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
            EventKind::RuleBased(rule) => {
                let month_day = Rule::month_day(rule).unwrap();
                month_day
            }
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

    pub fn from_str(month_day_string: &str) -> Result<Self, MonthDayParseError> {
        let month_day: Vec<&str> = month_day_string.split('-').collect();
        if month_day.len() != 2 {
            return Err(MonthDayParseError::InvalidFormat);
        };
        let month_string = month_day[0];
        let day_string = month_day[1];
        let month = match month_string.parse() {
            Ok(m) => m,
            Err(_) => return Err(MonthDayParseError::InvalidMonth),
        };
        if month < 1 || month > 12 {
            return Err(MonthDayParseError::InvalidMonth);
        }
        let day: u32 = match day_string.parse() {
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
            primary: primary.to_string().to_lowercase(),
            secondary: Some(secondary.to_string().to_lowercase()),
        }
    }

    pub fn from_primary(primary: &str) -> Self {
        Self {
            primary: primary.to_string().to_lowercase(),
            secondary: None,
        }
    }

    pub fn from_str(s: &str) -> Category {
        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() < 2 {
            Category {
                primary: parts[0].to_string().to_lowercase(),
                secondary: None,
            }
        } else {
            Category {
                primary: parts[0].to_string().to_lowercase(),
                secondary: Some(parts[1].to_string().to_lowercase()),
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

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

impl Weekday {
    pub fn as_chrono_weekday(&self) -> ChronoWeekday {
        match *self {
            Weekday::Monday => ChronoWeekday::Mon,
            Weekday::Tuesday => ChronoWeekday::Tue,
            Weekday::Wednesday => ChronoWeekday::Wed,
            Weekday::Thursday => ChronoWeekday::Thu,
            Weekday::Friday => ChronoWeekday::Fri,
            Weekday::Saturday => ChronoWeekday::Sat,
            Weekday::Sunday => ChronoWeekday::Sun,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, EnumString, Display)]
#[strum(ascii_case_insensitive)]
enum Ordinal {
    First = 1,
    Second = 2,
    Third = 3,
    Fourth = 4,
    Last = 5,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    ordinal: Ordinal,
    weekday: Weekday,
    month: Month,
}

#[derive(Debug, Display)]
pub enum RuleParseError {
    InvalidFormat,
    InvalidOrdinal,
    InvalidWeekday,
    InvalidMonth,
}

impl Rule {
    pub fn parse(rule_string: &str) -> Result<Self, RuleParseError> {
        let parts: Vec<String> = rule_string
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();

        if parts.len() != 4 {
            return Err(RuleParseError::InvalidFormat);
        }

        let ordinal = match Ordinal::from_str(&parts[0]) {
            Ok(ord) => ord,
            Err(_) => {
                return Err(RuleParseError::InvalidOrdinal);
            }
        };

        let weekday = match Weekday::from_str(&parts[1]) {
            Ok(wd) => wd,
            Err(_) => {
                return Err(RuleParseError::InvalidWeekday);
            }
        };

        if parts[2] != "in" && parts[2] != "of" {
            return Err(RuleParseError::InvalidFormat);
        }

        let month = match parts[3].parse::<Month>() {
            Ok(m) => m,
            Err(_) => {
                return Err(RuleParseError::InvalidMonth);
            }
        };

        Ok(Self {
            ordinal,
            weekday,
            month,
        })
    }

    pub fn resolve_date(&self, year: i32) -> Option<NaiveDate> {
        if self.ordinal == Ordinal::Last {
            last_weekday_in_month(year, self.month, self.weekday)
        } else {
            nth_weekday_in_month(year, self.month, self.weekday, self.ordinal)
        }
    }

    pub fn month_day(&self) -> Option<MonthDay> {
        if let Some(date) = self.resolve_date(self.year()) {
            Some(MonthDay {
                month: date.month(),
                day: date.day(),
            })
        } else {
            None
        }
    }

    pub fn year(&self) -> i32 {
        Local::now().year()
    }

    pub fn as_string(&self) -> String {
        format!("{} {} in {:?}", self.ordinal, self.weekday, self.month)
    }
}

fn nth_weekday_in_month(
    year: i32,
    month: Month,
    weekday: Weekday,
    ordinal: Ordinal,
) -> Option<NaiveDate> {
    let mut count = 0;
    for day in 1..=31 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month.number_from_month(), day) {
            if date.weekday() == weekday.as_chrono_weekday() {
                count += 1;
                if count == ordinal as i32 {
                    return Some(date);
                }
            }
        }
    }
    None
}

fn last_weekday_in_month(year: i32, month: Month, weekday: Weekday) -> Option<NaiveDate> {
    for day in (1..=31).rev() {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month.number_from_month(), day) {
            if date.weekday() == weekday.as_chrono_weekday() {
                return Some(date);
            }
        }
    }
    None
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
        let md = MonthDay::from_str("01-01").unwrap();
        assert_eq!(md.month(), 1);
        assert_eq!(md.day(), 1);

        let md = MonthDay::from_str("12-31").unwrap();
        assert_eq!(md.month(), 12);
        assert_eq!(md.day(), 31);

    let md = MonthDay::from_str("4-5").unwrap();
        assert_eq!(md.month(), 4);
        assert_eq!(md.day(), 5);    }

    #[test]
    fn month_day_from_invalid_string() {
        MonthDay::from_str("0101").unwrap_err();
        MonthDay::from_str("103").unwrap_err();
        MonthDay::from_str("13-01").unwrap_err();
        MonthDay::from_str("10-32").unwrap_err();
        MonthDay::from_str("02-30").unwrap_err();
        MonthDay::from_str("03-32").unwrap_err();
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

    #[test]
    fn last_weekday_in_month_function_works() {
        let date = last_weekday_in_month(2024, Month::January, Weekday::Monday).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 29).unwrap());

        let date = last_weekday_in_month(2024, Month::March, Weekday::Tuesday).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 26).unwrap());

        let date = last_weekday_in_month(2018, Month::August, Weekday::Thursday).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2018, 8, 30).unwrap());
    }

    #[test]
    fn nth_weekday_in_month_function_works() {
        let date =
            nth_weekday_in_month(2001, Month::January, Weekday::Monday, Ordinal::First).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2001, 1, 1).unwrap());

        let date =
            nth_weekday_in_month(1967, Month::March, Weekday::Tuesday, Ordinal::Second).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1967, 3, 14).unwrap());

        let date =
            nth_weekday_in_month(1999, Month::August, Weekday::Thursday, Ordinal::Third).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1999, 8, 19).unwrap());

        let date =
            nth_weekday_in_month(1987, Month::January, Weekday::Monday, Ordinal::Fourth).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1987, 1, 26).unwrap());
    }

    #[test]
    fn rule_parsing_works() {
        let rule = Rule::parse("first Monday in January").unwrap();
        assert_eq!(rule.ordinal, Ordinal::First);
        assert_eq!(rule.weekday, Weekday::Monday);
        assert_eq!(rule.month, Month::January);

        let rule = Rule::parse("second Tuesday of March").unwrap();
        assert_eq!(rule.ordinal, Ordinal::Second);
        assert_eq!(rule.weekday, Weekday::Tuesday);
        assert_eq!(rule.month, Month::March);

        let rule = Rule::parse("third Thursday in August").unwrap();
        assert_eq!(rule.ordinal, Ordinal::Third);
        assert_eq!(rule.weekday, Weekday::Thursday);
        assert_eq!(rule.month, Month::August);

        let rule = Rule::parse("fourth Monday in January").unwrap();
        assert_eq!(rule.ordinal, Ordinal::Fourth);
        assert_eq!(rule.weekday, Weekday::Monday);
        assert_eq!(rule.month, Month::January);

        let rule = Rule::parse("last Monday in January").unwrap();
        assert_eq!(rule.ordinal, Ordinal::Last);
        assert_eq!(rule.weekday, Weekday::Monday);
        assert_eq!(rule.month, Month::January);
    }

    #[test]
    fn rule_parsing_rejects_invalid_ordinal() {
        Rule::parse("firstest Monday in January").unwrap_err();
    }

    #[test]
    fn rule_parsing_rejects_invalid_weekday() {
        Rule::parse("first Monaday in January").unwrap_err();
    }

    #[test]
    fn rule_parsing_rejects_invalid_month() {
        Rule::parse("first Monday in Janury").unwrap_err();
    }

    #[test]
    fn rule_parsing_rejects_invalid_format() {
        Rule::parse("firstMondayinJanuary").unwrap_err();
        Rule::parse("first Monday January").unwrap_err();
        Rule::parse("Monday in January").unwrap_err();
    }

    #[test]
    fn rule_parsing_accepts_valid_rules() {
        let rule = Rule::parse("first Monday in January").unwrap();
        let date = rule.resolve_date(2024).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        let rule = Rule::parse("second Tuesday of March").unwrap();
        let date = rule.resolve_date(2024).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 12).unwrap());

        let rule = Rule::parse("third Thursday in August").unwrap();
        let date = rule.resolve_date(2024).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 8, 15).unwrap());

        let rule = Rule::parse("fourth Monday in January").unwrap();
        let date = rule.resolve_date(2024).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 22).unwrap());

        let rule = Rule::parse("last Monday in January").unwrap();
        let date = rule.resolve_date(2024).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 29).unwrap());
    }
}
