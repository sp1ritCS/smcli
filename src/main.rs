mod timetable;
mod credential;

use libschulmanager::{SmOfficeUser, Schulmanager, ClientAuthMethod};
use std::env::var;

#[macro_use]
extern crate clap;
use clap::App;

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

    let sm = match matches.value_of("AUTH").unwrap_or("invalid") {
        "o365" => {
            let (email, password) = credentials.get_office_keys();
            let user = SmOfficeUser {
                email: option_value("Email", matches.value_of("email"), "SM_EMAIL", email),
                password: option_value("Password", matches.value_of("password"), "SM_PASSWORD", password)
            };
            Schulmanager::login_office(user).await?
        },
        "jwt" => {
            let jwt_token = credentials.get_jwt_token();
            Schulmanager::new(ClientAuthMethod::JwtAuth(option_value("JWT Token", matches.value_of("jwt"), "SM_TOKEN", jwt_token))).await?
        }
        _ => {
            eprintln!("{} is an invalid authentication schema.\nValid schemas are: session, o365\n\nFor more info refer to the manpage", matches.value_of("AUTH").unwrap_or("invalid"));
            std::process::exit(1);
        }
    };

    match matches.subcommand() {
        ("timetable", Some(matches)) => {
            timetable::subcommand_timetable(matches, sm).await?;
        }
        _ => panic!("You did not provide a valid subcommand")
    }

    Ok(())
}
