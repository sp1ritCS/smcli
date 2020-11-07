use libschulmanager::{Schulmanager, SmTimetable, transformers::{smartv1::SmWeek,smartv2::{DayMap, Weekdays}}};
use clap::ArgMatches;
use chrono::{Datelike, Local};

enum Transformers {
    Legacy(Vec<SmWeek>),
    Smart(Vec<Weekdays>),
    SmartDM(Vec<DayMap>)
}

pub async fn subcommand_timetable(matches: &ArgMatches<'_>, sm: Schulmanager) -> Result<(), Box<dyn std::error::Error>>{
    let mut week: u32 = Local::today().iso_week().week();
    let mut year: i32 = Local::today().year();
    if matches.is_present("week") {
        week = matches.value_of("week").unwrap_or(&week.to_string()).parse().expect("given week is not a number")
    }
    if matches.is_present("year") {
        year = matches.value_of("year").unwrap_or(&year.to_string()).parse().expect("given year is not a number")
    }
    let table: SmTimetable = SmTimetable::new(sm, week, Some(year)).await?;
    let transtable = match matches.value_of("transformer").unwrap_or("smart") {
        "smart" => Transformers::Smart(table.to_smart_v2_weekdays()?),
        "smart_daymap" => Transformers::SmartDM(table.to_smart_v2_daymap()?),
        "legacy" => Transformers::Legacy(table.to_smart_v1()?),
        _ => {
            eprintln!("{} is not a valid transformer.\nValid transformers are: smart, legacy\n\nFor more info refer to the manpage", matches.value_of("transformer").unwrap_or("None"));
            std::process::exit(1);
        }
    };
    //let smart: Vec<timetable::SmWeek> = table.to_smart()?;
    match matches.value_of("output").unwrap_or("yaml") {
        "yaml" => {
            match transtable {
                Transformers::Smart(smart) => for s_timetable in smart {
                    println!("{}", serde_yaml::to_string(&s_timetable)?);
                },
                Transformers::SmartDM(smart) => for s_timetable in smart {
                    println!("{}", serde_yaml::to_string(&s_timetable)?);
                },
                Transformers::Legacy(legacy_smart) => for s_timetable in legacy_smart {
                    println!("{}", serde_yaml::to_string(&s_timetable)?);
                }
            }

        },
        "json" => match transtable {
            Transformers::Smart(smart) => println!("{}", serde_json::to_string(&smart)?),
            Transformers::SmartDM(smart) => println!("{}", serde_json::to_string(&smart)?),
            Transformers::Legacy(legacy_smart) => println!("{}", serde_json::to_string(&legacy_smart)?)
        },
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