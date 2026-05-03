use crate::{event::Event, filter::EventFilter};

mod text;
pub use text::TextFileProvider;

mod csv;
pub use csv::CSVFileProvider;

mod sqlite;
pub use sqlite::SQLiteProvider;

mod web;
pub use web::WebProvider;

pub trait EventProvider {
    fn name(&self) -> String;
    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>);
}
