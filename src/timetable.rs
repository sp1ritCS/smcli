use crate::AuthType;
use libschulmanager::{SmTimetable, timetable};
use clap::ArgMatches;
use chrono::{Datelike, Local};

pub async fn subcommand_timetable(matches: &ArgMatches<'_>, user: Option<AuthType>) -> Result<(), Box<dyn std::error::Error>>{
    let mut week: u32 = Local::today().iso_week().week();
    let mut year: i32 = Local::today().year();
    if matches.is_present("week") {
        week = matches.value_of("week").unwrap_or(&week.to_string()).parse().expect("given week is not a number")
    }
    if matches.is_present("year") {
        year = matches.value_of("year").unwrap_or(&year.to_string()).parse().expect("given year is not a number")
    }
    let table: SmTimetable = match user.unwrap() {
        AuthType::SESSION(schulmgr_session) => SmTimetable::from_user(schulmgr_session, week, Some(year)).await?,
        AuthType::O365(office_user) => SmTimetable::from_o365(office_user, week, Some(year)).await?
    };
    let smart: Vec<timetable::SmWeek> = table.to_smart()?;
    match matches.value_of("output").unwrap_or("yaml") {
        "yaml" => {
            for s_timetable in smart {
                println!("{}", serde_yaml::to_string(&s_timetable)?);
            }
        },
        "json" => println!("{}", serde_json::to_string(&smart)?),
        "curses" => {
            eprintln!("nCurses output not ready yet");
            std::process::exit(1);
        }
        _ => {
            eprintln!("{} is not a valid output type.\nValid types are: yaml, json, curses\n\nFor more info refer to the manpage", matches.value_of("output").unwrap_or("unkown"));
            std::process::exit(1);
        }
    }
    Ok(())
}