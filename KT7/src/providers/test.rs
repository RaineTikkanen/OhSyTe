use crate::event::{Category, Event};
use crate::providers::EventProvider;
use chrono::{NaiveDate};

pub struct TestProvider {
    name: String,
}

impl TestProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl EventProvider for TestProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, events: &mut Vec<Event>) {
        let mut new_events = vec![
            Event::new_singular(
                NaiveDate::from_ymd_opt(1759, 1, 15).unwrap(),
                String::from("The British Museum opens to the public."),
                Category::new("culture", "museum"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1892, 1, 15).unwrap(),
                String::from("James Naismith publishes the rules of basketball."),
                Category::new("sports", "basketball"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2001, 1, 15).unwrap(),
                String::from("Wikipedia is launched."),
                Category::new("technology", "internet"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1556, 1, 16).unwrap(),
                String::from("Philip II becomes King of Spain."),
                Category::new("politics", "monarchy"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1963, 1, 16).unwrap(),
                String::from("Top Gear host James May is born."),
                Category::new("culture", "television"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1917, 1, 17).unwrap(),
                String::from("The United States pays Denmark $25 million for the Virgin Islands."),
                Category::new("politics", "treaty"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1899, 1, 17).unwrap(),
                String::from("Al Capone is born."),
                Category::new("crime", "biography"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1956, 1, 17).unwrap(),
                String::from("Antero Mertaranta, Finnish sports journalist, is born."),
                Category::new("sports", "journalism"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1978, 1, 19).unwrap(),
                String::from(
                    "The last Volkswagen Beetle made in Germany leaves the production line.",
                ),
                Category::new("industry", "automotive"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1946, 1, 19).unwrap(),
                String::from("Dolly Parton is born."),
                Category::new("music", "biography"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1925, 1, 21).unwrap(),
                String::from("Albania declares itself a republic."),
                Category::new("politics", "statehood"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1981, 1, 21).unwrap(),
                String::from("Production of the DeLorean sports car begins in Northern Ireland."),
                Category::new("industry", "automotive"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1968, 1, 22).unwrap(),
                String::from("Apollo 5 lifts off."),
                Category::new("science", "spaceflight"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1959, 1, 22).unwrap(),
                String::from("Meiju Suvas is born."),
                Category::new("music", "biography"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1996, 1, 23).unwrap(),
                String::from("Java 1.0 released"),
                Category::new("technology", "programming"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1996, 1, 24).unwrap(),
                String::from("Air India Flight 101 crashes into Mont Blanc"),
                Category::new("aviation", "air crash"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1918, 1, 25).unwrap(),
                String::from(
                    "The Ukrainian People's Republic declares independence from Soviet Russia",
                ),
                Category::new("geography", "ukraine"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1956, 1, 26).unwrap(),
                String::from("Soviet Union cedes Porkkala back to Finland"),
                Category::new("geography", "finland"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2010, 1, 27).unwrap(),
                String::from("Apple announces the iPad."),
                Category::new("technology", "apple"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1956, 1, 28).unwrap(),
                String::from("Elvis Presley makes his first national television appearance."),
                Category::new("music", "television"),
            ),
            Event::new_singular(
                NaiveDate::from_ymd_opt(1965, 1, 29).unwrap(),
                String::from("Dominik Hašek, ice hockey goaltender, is born"),
                Category::new("sport", "ice hockey"),
            ),
        ];

        events.append(&mut new_events)
    }
}

#[cfg(test)]
mod tests {
    use crate::event::MonthDay;

    use super::*;

    #[test]
    fn name_provider() {
        let provider = TestProvider::new("test");
        assert_eq!(provider.name, "test")
    }

    #[test]
    fn events_provided() {
        let mut events: Vec<Event> = Vec::new();
        let provider = TestProvider::new("test");

        events.push(Event::new_singular(
            NaiveDate::from_ymd_opt(1999, 1, 1).unwrap(),
            String::from("TEST_EVENT"),
            Category::new("TESTING", "test"),
        ));

        provider.get_events(&mut events);

        let test_event_first = Event::new_singular(
            NaiveDate::from_ymd_opt(1999, 1, 1).unwrap(),
            String::from("TEST_EVENT"),
            Category::new("TESTING", "test"),
        );

        let test_event_second = Event::new_singular(
            NaiveDate::from_ymd_opt(1759, 1, 15).unwrap(),
            String::from("The British Museum opens to the public."),
            Category::new("culture", "museum"),
        );

        assert_eq!(events[0], test_event_first);
        assert_eq!(events[1], test_event_second);
        assert_eq!(events.len(), 22);
    }
}
