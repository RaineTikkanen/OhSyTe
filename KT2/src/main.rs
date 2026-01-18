fn format_date(date:i32) -> (i32,i32,i32){
    let date_string = date.to_string();

    let year_string = &date_string[..4];
    let year: i32 = year_string.parse().unwrap();

    let month_string = &date_string[4..6];
    let month: i32 = month_string.parse().unwrap();

    let day_string = &date_string[6..];
    let day: i32 = day_string.parse().unwrap();

    return (year,month,day)
}

fn print_events_on_date(events: [(i32, &str);18], date: (i32,i32)){

    println!("Events on date: {}.{}. :", date.1, date.0);


    let mut event_found = false;

    for event in events {
        let event_date = format_date(event.0);
        if date.0 == event_date.1 && date.1 == event_date.2 {
            event_found=true;
            println!("{}: {}", event_date.0, event.1);
        };
    };

    if !event_found {
        println!("No events found on date {}.{}.", date.1, date.0)
    }
}

fn main() {
    let today= (2026, 01, 17);

    let events = [
        (1759_01_15, " The British Museum opens to the public."),
        (1892_01_15, "James Naismith publishes the rules of basketball."),
        (2001_01_15, "Wikipedia is launched."),
        (1556_01_16, "Philip II becomes King of Spain."),
        (1963_01_16, "Top Gear host James May is born."),
        (1917_01_17, "The United States pays Denmark $25 million for the Virgin Islands."),
        (1899_01_17, "Al Capone is born."),
        (1956_01_17, "Antero Mertaranta, Finnish sports journalist, is born."),
        (2002_01_18, "The Sierra Leone Civil War is declared over."),
        (1639_01_19, "Hämeenlinna is granted city privileges."),
        (1978_01_19, "The last Volkswagen Beetle made in Germany leaves the production line."),
        (1946_01_19, "Dolly Parton is born."),
        (1841_01_20, "Hong Kong Island is occupied by the British during the First Opium War."),
        (1925_01_21, "Albania declares itself a republic."),
        (1981_01_21, "Production of the DeLorean sports car begins in Northern Ireland."),
        (1869_01_21, "Grigori Yefimovich \"Ra-Ra\" Rasputin, Russia's greatest love machine, is born."),
        (1968_01_22, "Apollo 5 lifts off"),
        (1959_01_22, "Meiju Suvas is born."),
    ];

    
    println!();

    print_events_on_date(events, (today.1, today.2));

    println!();

    for i in 15..23{
        println!();
        let date = (1, i);
        
        print_events_on_date(events, date);
    }


}
