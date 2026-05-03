use chrono::{Datelike, Local, NaiveDate};
use reqwest::{blocking::Client, blocking::Response};
use serde::Deserialize;
use serde_json;

use crate::event::{Category, Event, MonthDay};
use crate::filter::EventFilter;
use crate::providers::EventProvider;

pub struct WebProvider {
    name: String,
    url: String,
}

impl WebProvider {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct JSONEvent {
    category: String,
    date: String,
    description: String,
}

impl EventProvider for WebProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let today: NaiveDate = Local::now().date_naive();
        let month_day = MonthDay::new(today.month(), today.day());
        let date_parameter = format!("date={:02}-{:02}", month_day.month(), month_day.day());
        let client = Client::new();
        let url = format!("{}?{}", &self.url, date_parameter);
        let request = client.get(&url).send();
        let response: Response;
        if request.is_err() {
            eprintln!("Error while retrieving data: {:#?}", request.err());
            return;
        } else {
            response = request.ok().unwrap();
        }

        let json_events = response.json::<Vec<JSONEvent>>().unwrap();
        println!("Got {} events from JSON", json_events.len());

        for json_event in json_events {
            let date = NaiveDate::parse_from_str(&json_event.date, "%F").unwrap();
            let category = Category::from_str(&json_event.category);
            let event = Event::new_singular(date, json_event.description, category);
            events.push(event);
        }
    }
}
