const SECRET_SERVICE: &'static str = "smcli";
const CREDENTIAL_CONFIG: &'static str = "credential.yaml";

use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use keytar;
use clap::ArgMatches;
use std::env::var;

struct SecretService {
    pub account: String,
    pub password: String
}
impl SecretService {
    pub unsafe fn save(account: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(keytar::set_password(SECRET_SERVICE, account, password)?)
    }
    pub unsafe fn get(account: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let password = keytar::get_password(SECRET_SERVICE, account)?;
        if password.success {
            Ok(SecretService {
                account: String::from(account),
                password: String::from(password.password)
            })
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Could not read password from secret service")))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum PasswordStorage {
    SYSTEM,
    CONFIG(String)
}

#[derive(Serialize, Deserialize, Debug)]
struct SmStudent {
    pub id: usize,
    pub class_id: usize
}

#[derive(Serialize, Deserialize, Debug)]
struct OfficeCredentials {
    email: String,
    password: PasswordStorage
}
impl OfficeCredentials {
    pub fn new(email: String, password: Option<String>) -> Self {
        OfficeCredentials {
            email,
            password: match password {
                None => PasswordStorage::SYSTEM,
                Some(pw) => PasswordStorage::CONFIG(pw)
            }
        }
    }
    pub fn get_password(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(match &self.password {
            PasswordStorage::SYSTEM => unsafe {
                SecretService::get(&self.email)?.password
            },
            PasswordStorage::CONFIG(password) => password.to_string()
        })
    }
}

/* Currently unused */
#[derive(Serialize, Deserialize, Debug)]
struct SmCredentials {
    email: String,
    password: PasswordStorage
}

#[derive(Serialize, Deserialize, Debug)]
struct SmSession {
    pub session: String,
    pub session_sig: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CredentialConfig {
    student: Option<SmStudent>,
    office: Option<OfficeCredentials>,
    sm_user: Option<SmCredentials>,
    sm_session: Option<SmSession>
}
impl CredentialConfig {
    fn empty_creds() -> Self {
        CredentialConfig {
            student: None,
            office: None,
            sm_user: None,
            sm_session: None
        }
    }
    pub fn load() -> Self {
        match CredentialConfig::load_file() {
            Ok(creds) => creds,
            Err(_) => {
                eprintln!("Credential config not found, creating new one");
                let empty_creds = CredentialConfig::empty_creds();
                empty_creds.save().unwrap();
                empty_creds
            }
        }
    }
    fn load_file() -> Result<Self, Box<dyn std::error::Error>> {
        let dir = ProjectDirs::from("dev", "sp1rit", "smcli").expect("Unable to get path");
        let mut filepath = dir.config_dir().to_path_buf();
        filepath.push(CREDENTIAL_CONFIG);
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(filepath)?;
        let creds : CredentialConfig = serde_yaml::from_reader(file)?;
        Ok(creds)
    }
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = ProjectDirs::from("dev", "sp1rit", "smcli").expect("Unable to get path");
        let mut filepath = dir.config_dir().to_path_buf();
        std::fs::create_dir_all(&filepath)?;
        filepath.push(CREDENTIAL_CONFIG);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&filepath)?;
        Ok(serde_yaml::to_writer(file, &self)?)
    }

    pub fn update_student(&mut self, id: usize, class_id: usize) {
        let student = SmStudent {
            id,
            class_id
        };
        self.student = Some(student)
    }
    pub fn update_office(&mut self, email: String, password: String, store_in_plaintext: bool) {
        let office = match store_in_plaintext {
            true => OfficeCredentials::new(email, Some(password)),
            false => unsafe {
                SecretService::save(&email, &password).expect("unable to store the password. do you have your libsecret daemon started?");
                OfficeCredentials::new(email, None)
            }
        };
        self.office = Some(office)
    }
    pub fn update_session(&mut self, session: String, session_sig: String) {
        let session = SmSession {
            session,
            session_sig
        };
        self.sm_session = Some(session)
    }

    pub fn get_student_keys(self) -> (Option<usize>, Option<usize>) {
        match self.student {
            Some(student) => (Some(student.id), Some(student.class_id)),
            None => (None, None)
        }
    }
    pub fn get_office_keys(&self) -> (Option<String>, Option<String>) {
        match &self.office {
            Some(office) => (Some(office.email.to_string()), match &office.get_password() {
                Ok(password) => Some(password.to_string()),
                Err(_) => None
            }),
            None => (None, None)
        }
    }
    pub fn get_session_keys(&self) -> (Option<String>, Option<String>) {
        match &self.sm_session {
            Some(session) => (Some(session.session.to_string()), Some(session.session_sig.to_string())),
            None => (None, None)
        }
    }
}

fn option_value(param: Option<&str>, env: &str) -> Option<String> {
    match param {
        Some(val) => Some(String::from(val)),
        None => {
            match var(env) {
                Ok(val) => Some(val),
                Err(_) => None
            }
        }
    }
}

pub fn subcommand_credentials(matches: &ArgMatches<'_>, submatches: &ArgMatches<'_>, credentials: &mut CredentialConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(id), Some(class_id)) = (option_value(matches.value_of("id"), "SM_ID"), option_value(matches.value_of("class_id"), "SM_CLASS_ID")) {
        credentials.update_student(id.parse()?, class_id.parse()?)
    }
    if let (Some(email), Some(password)) = (option_value(matches.value_of("email"), "SM_EMAIL"), option_value(matches.value_of("password"), "SM_PASSWORD")) {
        credentials.update_office(String::from(email), String::from(password), submatches.is_present("no_secret"));
    }
    if let (Some(session), Some(session_sig)) = (option_value(matches.value_of("session"), "SM_SESSION"), option_value(matches.value_of("session_sig"), "SM_SESSION_SIG")) {
        credentials.update_session(String::from(session), String::from(session_sig));
    }
    // TODO: This appears to be overwriting the existing file, but it has issues with fs compression/different alignments. Maybe delete the old config first and then save?
    credentials.save()
}