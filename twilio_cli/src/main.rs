use std::process;

use inquire::{validator::Validation, Text};
use twilio_cli::{choose_action, choose_resource, request_credentials, Action};
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
    let account = twilio
        .get_account(None)
        .unwrap_or_else(|error| panic!("{}", error));

    println!(
        "✅ Account details good! {} ({} - {})",
        account.friendly_name, account.type_field, account.status
    );

    loop {
        let sub_resource = choose_resource();

        loop {
            let action = choose_action();

            match action {
                Action::GetAccount => {
                    let account_sid = Text::new("Please provide an account SID:")
                        .with_placeholder("AC...")
                        .with_validator(|val: &str| match val.starts_with("AC") {
                            true => Ok(Validation::Valid),
                            false => {
                                Ok(Validation::Invalid("Account SID must start with AC".into()))
                            }
                        })
                        .with_validator(|val: &str| match val.len() {
                            34 => Ok(Validation::Valid),
                            _ => Ok(Validation::Invalid(
                                "Your SID should be 34 characters in length".into(),
                            )),
                        })
                        .prompt()
                        .unwrap();
                    let account = twilio
                        .get_account(Some(&account_sid))
                        .unwrap_or_else(|error| panic!("{}", error));
                    println!("{:?}", account);
                }
                Action::CreateAccount => {
                    let friendly_name = Text::new("Enter a friendly name (empty for default):")
                        .prompt()
                        .unwrap();

                    let account = twilio
                        .create_account(Some(&friendly_name))
                        .unwrap_or_else(|error| panic!("{}", error));
                    println!(
                        "Account created: {} ({})",
                        account.friendly_name, account.sid
                    );
                }
                Action::ListAccounts => {
                    println!("Retrieving accounts...");
                    let friendly_name = Text::new("Search by friendly name? (empty for none):")
                        .prompt()
                        .unwrap();
                    let status = Text::new("Filter by status?").prompt().unwrap();
                    let mut accounts = twilio
                        .list_accounts(Some(&friendly_name), None)
                        .unwrap_or_else(|error| panic!("{}", error));

                    for i in accounts.iter_mut() {
                        println!("Account {} ({})", i.friendly_name, i.sid);
                    }
                }
                Action::Back => break,
                Action::Exit => process::exit(0),
            }
        }
    }
}
