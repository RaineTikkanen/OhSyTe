use crate::event::Event;

mod text;
pub use text::TextFileProvider;

mod test;
pub use test::TestProvider;

mod csv;
pub use csv::CSVFileProvider;

pub trait EventProvider {
    fn name(&self) -> String;
    fn get_events(&self, events: &mut Vec<Event>);
}
