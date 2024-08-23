/*! Twilly_cli is a CLI tool for interacting the Twilio API.

Coverage is partial yet offers a user-friendly way to interact
with Twilio via the terminal. The CLI currently covers:

- Accounts
- Conversations

This crate has been developed alongside the `twilly` crate which backs
the functionality of the crate.

# Features

- Local _profile_ memory to avoid repeatedly providing Twilio credentials.
- Intuitive account & conversation related controls.
- Additional _helpers_ not found in the default Twilio CLI.

*/
use std::{fmt::Display, process};

use chrono::Datelike;
use chrono::NaiveDate;
use inquire::MultiSelect;
use inquire::{
    validator::Validation, Confirm, DateSelect, InquireError, Password, PasswordDisplayMode,
    Select, Text,
};
use twilly::TwilioConfig;

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

/// A wrapper around the Inquire crates various input controls. This is used
/// to abstract the prompting and handling errors or cancellations.
pub trait InquireControl<T> {
    fn prompt_user(&self) -> Result<T, InquireError>;
}

impl InquireControl<String> for Text<'_> {
    fn prompt_user(&self) -> Result<String, InquireError> {
        self.clone().prompt()
    }
}

impl InquireControl<String> for Password<'_> {
    fn prompt_user(&self) -> Result<String, InquireError> {
        self.clone().prompt()
    }
}

impl InquireControl<bool> for Confirm<'_> {
    fn prompt_user(&self) -> Result<bool, InquireError> {
        self.clone().prompt()
    }
}

impl InquireControl<NaiveDate> for DateSelect<'_> {
    fn prompt_user(&self) -> Result<NaiveDate, InquireError> {
        self.clone().prompt()
    }
}

// Examines an error from Inquire to determine the cause. If the user
// canceled an operation (pressed ESC) then the program returns. All
// other errors are determined fatal and will terminate the program
// through a panic or exit.
fn handle_inquire_error<T>(error: InquireError) -> Option<T> {
    match error {
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
    }
}

/// Prompts the user for some sort of input. Takes any function that
/// implements the `InquireControl` trait and returns the output
/// from the user. If `None` is returned it is assumed the user
/// un-forcefully cancelled the action, e.g. pressed ESC.
pub fn prompt_user<T>(control: impl InquireControl<T>) -> Option<T> {
    match control.prompt_user() {
        Ok(result) => Some(result),
        Err(error) => handle_inquire_error(error),
    }
}

/// Prompts the user a selection from the provided options. Takes
/// any form of Inquires Select and returns the output
/// from the user. If `None` is returned it is assumed the user
/// un-forcefully cancelled the action, e.g. pressed ESC.
///
/// This has the same pattern as `prompt_user` for obvious reasons.
pub fn prompt_user_selection<T: Display>(control: Select<'_, T>) -> Option<T> {
    match control.prompt() {
        Ok(result) => Some(result),
        Err(error) => handle_inquire_error(error),
    }
}

pub fn prompt_user_multi_selection<T: Display>(control: MultiSelect<'_, T>) -> Option<Vec<T>> {
    match control.prompt() {
        Ok(result) => Some(result),
        Err(error) => handle_inquire_error(error),
    }
}

/// The options available to filter search results.
pub enum FilterChoice {
    /// Any option, not limited to anything.
    Any,
    /// One of the provided options, dependant on application state.
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
    let filter_choice_opt = prompt_user_selection(filter_choice_prompt);

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

/// The possible actions a user may make.
pub enum ActionChoice {
    /// Navigate backwards in the menu.
    Back,
    /// Exit the application completely.
    Exit,
    /// An option provided, dependent on the state of the application.
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
    let action_choice_opt = prompt_user_selection(action_choice_prompt);

    match action_choice_opt {
        Some(action_choice) => match action_choice.as_str() {
            "Back" => Some(ActionChoice::Back),
            "Exit" => Some(ActionChoice::Exit),
            _ => Some(ActionChoice::Other(action_choice)),
        },
        None => None,
    }
}

pub struct DateRange {
    pub minimum_date: chrono::NaiveDate,
    pub maximum_date: chrono::NaiveDate,
}

pub fn get_date_from_user(
    message: &str,
    date_range: Option<DateRange>,
) -> Option<chrono::NaiveDate> {
    let selected_date = match date_range {
        Some(date_range) => {
            let date_selection_prompt = DateSelect::new(message)
                .with_min_date(
                    chrono::NaiveDate::from_ymd_opt(
                        date_range.minimum_date.year(),
                        date_range.minimum_date.month(),
                        date_range.minimum_date.day(),
                    )
                    .unwrap(),
                )
                .with_max_date(
                    chrono::NaiveDate::from_ymd_opt(
                        date_range.maximum_date.year(),
                        date_range.maximum_date.month(),
                        date_range.maximum_date.day(),
                    )
                    .unwrap(),
                )
                .with_week_start(chrono::Weekday::Mon);

            prompt_user(date_selection_prompt)
        }
        None => {
            let date_selection_prompt =
                DateSelect::new(message).with_week_start(chrono::Weekday::Mon);
            prompt_user(date_selection_prompt)
        }
    };

    selected_date
}
