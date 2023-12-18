use std::process;

use twilio_cli::request_credentials;
use twilio_rust;

fn main() {
    println!("Welcome to Twilio Rust! I'm here to help you interact with Twilio!");

    let config = request_credentials().unwrap_or_else(|err| match err {
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
    });
    let twilio = twilio_rust::Client::new(config);

    println!("Checking account...");
    let account = twilio.get_account();
    match account {
        Ok(account) => {
            println!(
                "✅ Account details good! {} ({} - {})",
                account.friendly_name, account.type_field, account.status
            );
        }
        Err(err) => {
            panic!("Account check failed: {}", err)
        }
    }
}
