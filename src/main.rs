use libschulmanager::{SmUser, SmTimetable, timetable};
use chrono::{Datelike, Local};

#[macro_use]
extern crate clap;
use clap::App;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>>{
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("smcli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    #[allow(unused_assignments)]
    let mut user: Option<SmUser> = None;
    match matches.value_of("AUTH").unwrap_or("invalid") {
        "session" => {
            user = Some(SmUser {
                session: String::from(matches.value_of("session").expect("session is required for session auth\n\nFor more info refer to the manpage")),
                session_sig: String::from(matches.value_of("session_sig").expect("session_sig is required for session auth\n\nFor more info refer to the manpage")),
                student_id: String::from(matches.value_of("id").expect("id is required for session auth\n\nFor more info refer to the manpage")).parse().expect("id is not a valid number"),
                student_class_id: String::from(matches.value_of("class_id").expect("class_id is required for session auth\n\nFor more info refer to the manpage")).parse().expect("class_id is not a valid number")
            });
        },
        "o365" => {
            eprintln!("The O365 Schema is not ready yet");
            std::process::exit(1);
        },
        _ => {
            eprintln!("{} is an invalid authentication schema.\nValid schemas are: session, o365\n\nFor more info refer to the manpage", matches.value_of("AUTH").unwrap_or("invalid"));
            std::process::exit(1);
        }
    }

    if let Some(matches) = matches.subcommand_matches("timetable") {
        let mut week: u32 = Local::today().iso_week().week();
        if matches.is_present("week") {
            week = matches.value_of("week").unwrap_or(&week.to_string()).parse().expect("given week is not a number")
        }
        let table: SmTimetable = SmTimetable::new(user.expect("Invalid Authentication Root"), week).await?;
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
    }

    Ok(())
}
