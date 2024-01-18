use std::process;

use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Select, Text};
use twilio_rust::TwilioConfig;

/// Requests Twilio Account SID and auth token pair from the user and returns
/// it as a `TwilioConfig` struct.
pub fn request_credentials() -> TwilioConfig {
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
        .prompt()
        .unwrap_or_else(|err| {
            panic_inquire_error(err);
            "".into()
        });

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
        .prompt()
        .unwrap_or_else(|err| {
            panic_inquire_error(err);
            "".into()
        });

    TwilioConfig::build(account_sid, auth_token)
}

fn panic_inquire_error(error: InquireError) {
    match error {
        inquire::InquireError::OperationCanceled | inquire::InquireError::OperationInterrupted => {
            eprintln!("Operation was cancelled or interrupted. Closing program.");
            process::exit(130);
        }
        inquire::InquireError::IO(err) => {
            panic!("Unhandled IO Error: {}", err);
        }
        inquire::InquireError::NotTTY => {
            panic!("Unable to handle non-TTY input device.");
        }
        inquire::InquireError::InvalidConfiguration(err) => {
            panic!(
                "Invalid configuration for select, multi_select, or date_select: {}",
                err
            );
        }
        inquire::InquireError::Custom(err) => {
            panic!(
                "Custom user error caught at root. This probably shouldn't have happened :/ {}",
                err
            );
        }
    }
}

pub enum FilterChoice {
    Any,
    Other(String),
}

/// Gets the choice of a filter from options provided as an argument. An `Any` option will be
/// presented also suggesting that no specific filter is required.
///
/// This will return `Any` if the user selected this option or the `String` of the
/// user's choice.
pub fn get_filter_choice_from_user(mut filter_options: Vec<String>, message: &str) -> FilterChoice {
    filter_options.insert(0, String::from("Any"));
    let filter_choice = Select::new(message, filter_options).prompt().unwrap();

    if filter_choice.as_str() == "Any" {
        FilterChoice::Any
    } else {
        FilterChoice::Other(filter_choice)
    }
}

pub enum ActionChoice {
    Back,
    Exit,
    Other(String),
}

/// Gets the choice of an action from options provided as arguments. `Back` and `Exit` options
/// will be presented also allowing the user to navigate backwards in a menu or exit.
///
/// This will return an enum of either the back or exit options, otherwise the string
/// of the user's choice.
pub fn get_action_choice_from_user(mut action_options: Vec<String>, message: &str) -> ActionChoice {
    let mut back_and_exit_options = vec![String::from("Back"), String::from("Exit")];
    action_options.append(&mut back_and_exit_options);

    let action_choice = Select::new(message, action_options).prompt().unwrap();

    match action_choice.as_str() {
        "Back" => return ActionChoice::Back,
        "Exit" => return ActionChoice::Exit,
        _ => return ActionChoice::Other(action_choice),
    }
}
