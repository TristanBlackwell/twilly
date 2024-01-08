use std::process;

use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Text};
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
