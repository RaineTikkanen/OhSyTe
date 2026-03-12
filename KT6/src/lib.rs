use std::error::Error;

mod birthday;
mod event;
mod providers;

use birthday::handle_birthday;
use chrono::{Datelike, Local, NaiveDate};
use event::{Category, Event, MonthDay};
use providers::{EventProvider, TestProvider};

pub fn run() -> Result<(), Box<dyn Error>> {
    handle_birthday();

    let mut events: Vec<Event> = Vec::new();

    let today: NaiveDate = Local::now().date_naive();
    let today_month_day = MonthDay::new(today.month(), today.day());

    let test_provider = TestProvider::new("test");
    test_provider.get_events(&mut events);

    for event in events {
        println!("{}", event);
    }

    Ok(())
}
