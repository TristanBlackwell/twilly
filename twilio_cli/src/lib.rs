use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Text};
use twilio_rust::TwilioConfig;

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
