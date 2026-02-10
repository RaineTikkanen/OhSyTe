use chrono::{NaiveDate, Local, DateTime, Datelike};


fn main() {

    let env_birthday = match std::env::var("BIRTHDAY") {
        Ok(b)=> b,
        Err(_)=>{
            println!("BIRTHDAY environment variable not found");
            std::process::exit(1);
        }
    };

    let date_format= "%Y-%m-%d";
    let date_print_format = "%d.%m.%Y";
    let birthday_parse_result= NaiveDate::parse_from_str(&env_birthday, date_format);

    let birthday = match birthday_parse_result{
        Ok(b)=>b,
        Err(_)=>{
            println!("BIRTHDAY environment variable not valid. Please set variable as YEAR-MONTH-DAY");
            std::process::exit(1);
        },
    };

    let today=Local::now().date_naive();
    let age_in_days=(today-birthday).num_days();

    if age_in_days < 0 {
        println!("Are you from the future?");
        std::process::exit(0);
    }

    if (birthday.day() == today.day()) && (birthday.month() == today.month()){
        println!("Today is your birthday! Congratulations!");
    }

    println!("You are {} days old", age_in_days);

    if age_in_days % 1000 == 0 {
        println!("Thats a nice round number");
    }


}