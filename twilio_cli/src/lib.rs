use std::{fmt::Display, process};

use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Select, Text};
use twilio_rust::TwilioConfig;

/// Requests Twilio Account SID and auth token pair from the user and returns
/// it as a `TwilioConfig` struct.
pub fn request_credentials() -> TwilioConfig {
    let account_sid_prompt = Text::new("Please provide an account SID:")
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
        });
    let account_sid = prompt_user(account_sid_prompt).unwrap_or(String::from(""));

    let auth_token_prompt = Password::new("Provide the auth token (input hidden):")
        .with_validator(|val: &str| match val.len() {
            32 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(
                "Your SID should be 32 characters in length".into(),
            )),
        })
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_display_toggle_enabled()
        .without_confirmation()
        .with_help_message("Input is masked. Use Ctrl + R to toggle visibility.");
    let auth_token = prompt_user(auth_token_prompt).unwrap_or(String::from(""));

    TwilioConfig::build(account_sid, auth_token)
}

trait InquireControl<T> {
    fn prompt_user(&self) -> Result<T, InquireError>;
}

impl InquireControl<String> for Text<'_> {
    fn prompt_user(&self) -> Result<String, InquireError> {
        self.prompt()
    }
}

impl InquireControl<String> for Password<'_> {
    fn prompt_user(&self) -> Result<String, InquireError> {
        self.prompt()
    }
}

impl<T: Display + Copy> InquireControl<T> for Select<'_, T> {
    fn prompt_user(&self) -> Result<T, InquireError> {
        self.prompt()
    }
}

/// Prompts the user for some sort of input. Takes any function that
/// implements the `InquireControl` trait and returns the output
/// from the user. If `None` is returned it is assumed the user
/// un-forcefully cancelled the action, e.g. pressed ESC.
pub fn prompt_user<T>(control: impl InquireControl<T>) -> Option<T> {
    match control.prompt_user() {
        Ok(result) => Some(result),
        // Examines an error from Inquire to determine the cause. If the user
        // canceled an operation (pressed ESC) then the program returns. All
        // other errors are determined fatal and will terminate the program
        // through a panic or exit.
        Err(error) => match error {
            inquire::InquireError::OperationCanceled => None,
            inquire::InquireError::OperationInterrupted => {
                eprintln!("Operation interrupted. Closing program.");
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
        },
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
pub fn get_filter_choice_from_user(
    mut filter_options: Vec<String>,
    message: &str,
) -> Option<FilterChoice> {
    filter_options.insert(0, String::from("Any"));
    let filter_choice_prompt = Select::new(message, filter_options);
    let filter_choice_opt = prompt_user(filter_choice_prompt);

    if filter_choice_opt.is_some() {
        let filter_choice = filter_choice_opt.unwrap();
        if filter_choice.as_str() == "Any" {
            Some(FilterChoice::Any)
        } else {
            Some(FilterChoice::Other(filter_choice))
        }
    } else {
        None
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
pub fn get_action_choice_from_user(
    mut action_options: Vec<String>,
    message: &str,
) -> Option<ActionChoice> {
    let mut back_and_exit_options = vec![String::from("Back"), String::from("Exit")];
    action_options.append(&mut back_and_exit_options);

    let action_choice_prompt = Select::new(message, action_options);
    let action_choice_opt = prompt_user(action_choice_prompt);
    let action_choice = action_choice_opt.unwrap();

    match action_choice.as_str() {
        "Back" => return Some(ActionChoice::Back),
        "Exit" => return Some(ActionChoice::Exit),
        _ => return Some(ActionChoice::Other(action_choice)),
    }
}
