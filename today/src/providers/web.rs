use crate::event::{Category, Event};
use crate::filter::EventFilter;
use crate::providers::{EventProvider, EventProviderError};
use chrono::NaiveDate;
use log::{error, info};
use reqwest::{blocking::Client, blocking::Response};
use serde::Deserialize;

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
        let date = match filter.month_day() {
            Some(d) => {
                format!("date={:02}-{:02}", d.month(), d.day())
            }
            None => {
                error!("No date in filter. Can not get events from web source");
                return;
            }
        };

        let client = Client::new();
        let url = format!("{}?{}", &self.url, date);
        let request = client.get(&url).send();
        let response: Response;
        if request.is_err() {
            error!("Error while retrieving data: {:#?}", request.err());
            return;
        } else {
            response = match request.ok() {
                Some(r) => r,
                None => {
                    info! {"Got empty response from web provider '{}'", self.name()};
                    return;
                }
            };
        }

        let json_events = match response.json::<Vec<JSONEvent>>() {
            Ok(j) => j,
            Err(e) => {
                error!("Error while parsing JSON: {:#?}", e);
                return;
            }
        };
        info!(
            "Got {} events from web provider '{}'",
            json_events.len(),
            self.name()
        );

        for json_event in json_events {
            let date = NaiveDate::parse_from_str(&json_event.date, "%F").unwrap();
            let category = Category::from_str(&json_event.category);
            let event = Event::new_singular(date, json_event.description, category);
            if filter.accepts(&event) {
                events.push(event);
            }
        }
    }

    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        return Err(EventProviderError::OperationNotSupported);
    }
}
