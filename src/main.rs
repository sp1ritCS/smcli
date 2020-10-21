mod timetable;
mod credential;

use libschulmanager::{SmUser, SmOfficeUser};
use std::env::var;

#[macro_use]
extern crate clap;
use clap::App;

pub enum AuthType {
    SESSION(SmUser),
    O365(SmOfficeUser)
}

fn option_value(name: &str, param: Option<&str>, env: &str, key: Option<String>) -> String {
    match param {
        Some(val) => String::from(val),
        None => match var(env) {
            Ok(val) => val,
            Err(_) => match key {
                Some(val) => val,
                None => panic!("The value for {} is missing. Consider setting the env var {} or refer to the man page", name, env)
            }
        }
    }
}
fn option_value_int(name: &str, param: Option<&str>, env: &str, key: Option<usize>) -> usize {
    match param {
        Some(val) => String::from(val).parse().expect(&format!("{} is not a valid number", name)),
        None => match var(env) {
            Ok(val) => val.parse().expect(&format!("{} is not a valid number", name)),
            Err(_) => match key {
                Some(val) => val,
                None => panic!("The value for {} is missing. Consider setting the env var {} or refer to the man page", name, env)
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>>{
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("smcli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut credentials = credential::CredentialConfig::load();

    /* the credential subcommand has to be executed before the getting the user */
    if let Some(submatches) = matches.subcommand_matches("credential") {
        credential::subcommand_credentials(&matches, &submatches, &mut credentials)?;
        std::process::exit(0);
    }

    #[allow(unused_assignments)]
    let mut user: Option<AuthType> = None;
    match matches.value_of("AUTH").unwrap_or("invalid") {
        "session" => {
            let (session, session_sig) = credentials.get_session_keys();
            let (id, class_id) = credentials.get_student_keys();
            user = Some(AuthType::SESSION(SmUser {
                session: option_value("Session", matches.value_of("session"), "SM_SESSION", session),
                session_sig: option_value("Session sig", matches.value_of("session_sig"), "SM_SESSION_SIG", session_sig),
                student_id: option_value_int("Id", matches.value_of("id"), "SM_ID", id),
                student_class_id: option_value_int("Class Id", matches.value_of("class_id"), "SM_CLASS_ID", class_id)
            }));
        },
        "o365" => {
            let (email, password) = credentials.get_office_keys();
            let (id, class_id) = credentials.get_student_keys();
            user = Some(AuthType::O365(SmOfficeUser {
                email: option_value("Email", matches.value_of("email"), "SM_EMAIL", email),
                password: option_value("Password", matches.value_of("password"), "SM_PASSWORD", password),
                student_id: option_value_int("Id", matches.value_of("id"), "SM_ID", id),
                student_class_id: option_value_int("Class Id", matches.value_of("class_id"), "SM_CLASS_ID", class_id)
            }));
        },
        _ => {
            eprintln!("{} is an invalid authentication schema.\nValid schemas are: session, o365\n\nFor more info refer to the manpage", matches.value_of("AUTH").unwrap_or("invalid"));
            std::process::exit(1);
        }
    }

    match matches.subcommand() {
        ("timetable", Some(matches)) => {
            timetable::subcommand_timetable(matches, user).await?;
        }
        _ => panic!("You did not provide a valid subcommand")
    }

    Ok(())
}
