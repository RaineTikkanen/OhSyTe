#[derive(Debug, PartialEq, Copy, Clone)]
enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Debug, PartialEq)]
struct MonthDay {
    month: Month,
    day: u8,
}

#[derive(Debug, PartialEq)]
struct Date {
    year: i16,
    month: Month,
    day: u8,
}
impl Date {
    fn new(year: i16, month: Month, day: u8) -> Self {
        Self { year, month, day }
    }
    fn month_day(&self) -> MonthDay {
        MonthDay {
            month: self.month,
            day: self.day,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Category {
    primary: String,
    secondary: Option<String>,
}
impl Category {
    fn new(primary: &str, secondary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: Some(secondary.to_string()),
        }
    }
    fn from_primary(primary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: None,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Event {
    date: Date,
    description: String,
    category: Category,
}
impl Event {
    fn new(date: Date, description: String, category: Category) -> Self {
        Self {
            date,
            description,
            category,
        }
    }
}


fn print_events_on_date(events: &Vec<Event>, date: Date) {

    let day_ending = match date.day {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ =>"th"
    };

    println!("Events on date: {:?} {}{}", date.month, date.day, day_ending);

    let mut event_found = false;

    for event in events {
        if date.month_day() == event.date.month_day(){
            event_found = true;
            println!("{}: {}", event.date.year, event.description);
        };
    }

    if !event_found {
        println!("No events found on date {:?} {}{}", date.month, date.day, day_ending)
    }
}

fn main() {
    let events = vec![
        Event::new(
            Date::new(1759, Month::January, 15),
            String::from("The British Museum opens to the public."),
            Category::new("culture", "museum"),
        ),
        Event::new(
            Date::new(1892, Month::January, 15),
            String::from("James Naismith publishes the rules of basketball."),
            Category::new("sports", "basketball"),
        ),
        Event::new(
            Date::new(2001, Month::January, 15),
            String::from("Wikipedia is launched."),
            Category::new("technology", "internet"),
        ),
        Event::new(
            Date::new(1556, Month::January, 16),
            String::from("Philip II becomes King of Spain."),
            Category::new("politics", "monarchy"),
        ),
        Event::new(
            Date::new(1963, Month::January, 16),
            String::from("Top Gear host James May is born."),
            Category::new("culture", "television"),
        ),
        Event::new(
            Date::new(1917, Month::January, 17),
            String::from("The United States pays Denmark $25 million for the Virgin Islands."),
            Category::new("politics", "treaty"),
        ),
        Event::new(
            Date::new(1899, Month::January, 17),
            String::from("Al Capone is born."),
            Category::new("crime", "biography"),
        ),
        Event::new(
            Date::new(1956, Month::January, 17),
            String::from("Antero Mertaranta, Finnish sports journalist, is born."),
            Category::new("sports", "journalism"),
        ),
        Event::new(
            Date::new(2002, Month::January, 18),
            String::from("The Sierra Leone Civil War is declared over."),
            Category::new("history", "war"),
        ),
        Event::new(
            Date::new(1639, Month::January, 19),
            String::from("Hämeenlinna is granted city privileges."),
            Category::new("history", "cities"),
        ),
        Event::new(
            Date::new(1978, Month::January, 19),
            String::from("The last Volkswagen Beetle made in Germany leaves the production line."),
            Category::new("industry", "automotive"),
        ),
        Event::new(
            Date::new(1946, Month::January, 19),
            String::from("Dolly Parton is born."),
            Category::new("music", "biography"),
        ),
        Event::new(
            Date::new(1841, Month::January, 20),
            String::from("Hong Kong Island is occupied by the British during the First Opium War."),
            Category::new("history", "colonialism"),
        ),
        Event::new(
            Date::new(1925, Month::January, 21),
            String::from("Albania declares itself a republic."),
            Category::new("politics", "statehood"),
        ),
        Event::new(
            Date::new(1981, Month::January, 21),
            String::from("Production of the DeLorean sports car begins in Northern Ireland."),
            Category::new("industry", "automotive"),
        ),
        Event::new(
            Date::new(1869, Month::January, 21),
            String::from("Grigori Yefimovich \"Ra-Ra\" Rasputin, Russia's greatest love machine, is born."),
            Category::new("history", "biography"),
        ),
        Event::new(
            Date::new(1968, Month::January, 22),
            String::from("Apollo 5 lifts off."),
            Category::new("science", "spaceflight"),
        ),
        Event::new(
            Date::new(1959, Month::January, 22),
            String::from("Meiju Suvas is born."),
            Category::new("music", "biography"),
        ),
        Event::new(
            Date::new(1996, Month::January, 23),
            String::from("Java 1.0 released"),
            Category::new("technology", "programming"),
        ),
        Event::new(
            Date::new(1996, Month::January, 24),
            String::from("Air India Flight 101 crashes into Mont Blanc"),
            Category::new("aviation", "air crash"),
        ),
        Event::new(
            Date::new(1918, Month::January, 25),
            String::from("The Ukrainian People's Republic declares independence from Soviet Russia"),
            Category::new("geography", "ukraine"),
        ),
        Event::new(
            Date::new(1956, Month::January, 26),
            String::from("Soviet Union cedes Porkkala back to Finland"),
            Category::new("geography", "finland"),
        ),
        Event::new(
            Date::new(2010, Month::January, 27),
            String::from("Apple announces the iPad."),
            Category::new("technology", "apple"),
        ),
        Event::new(
            Date::new(1956, Month::January, 28),
            String::from("Elvis Presley makes his first national television appearance."),
            Category::new("music", "television"),
        ),
        Event::new(
            Date::new(1965, Month::January, 29),
            String::from("Dominik Hašek, ice hockey goaltender, is born"),
            Category::new("sport", "ice hockey"),
        ),
    ];

    let today = Date::new(2026, Month::January, 21);

    for i in 15..30 {
        let date = Date::new(2026, Month::January, i);
        print_events_on_date(&events, date);
        println!();
    }


    print_events_on_date(&events, today);


}
