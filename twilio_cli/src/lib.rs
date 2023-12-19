use std::{fmt, str::FromStr};

use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Select, Text};
use twilio_rust::{SubResource, TwilioConfig};

pub fn request_credentials() -> Result<TwilioConfig, InquireError> {
    let account_sid = Text::new("Please provide an account SID:")
        .with_placeholder("AC...")
        .with_validator(|val: &str| match val.starts_with("AC") {
            true => Ok(Validation::Valid),
            false => Ok(Validation::Invalid("Account SID must start with AC".into())),
        })
        .with_validator(|val: &str| match val.len() {
            34 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(
                "Your SID should be 34 characters in length".into(),
            )),
        })
        .prompt()?;

    let auth_token = Password::new("Provide the auth token (input hidden):")
        .with_validator(|val: &str| match val.len() {
            32 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(
                "Your SID should be 32 characters in length".into(),
            )),
        })
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_display_toggle_enabled()
        .without_confirmation()
        .with_help_message("Input is masked. Use Ctrl + R to toggle visibility.")
        .prompt()?;

    Ok(TwilioConfig::build(account_sid, auth_token))
}

pub fn choose_resource() -> SubResource {
    let options = vec!["Account", "Sync"];
    let sub_resource = Select::new("Select a resource:", options).prompt();

    SubResource::from_str(sub_resource.unwrap()).unwrap()
}

pub enum Action {
    GetAccount,
    CreateAccount,
}

impl FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Get account" => Ok(Action::GetAccount),
            "Create account" => Ok(Action::CreateAccount),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn choose_action() -> Action {
    let options = vec!["Get account", "Create account"];
    let action = Select::new("Select a action:", options).prompt();

    Action::from_str(action.unwrap()).unwrap()
}
