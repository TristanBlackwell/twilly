use std::process;

use inquire::{validator::Validation, Select, Text};
use strum::IntoEnumIterator;
use twilio_cli::{choose_action, choose_resource, request_credentials, Action};
use twilio_rust::{self, account::Status, TwilioConfig};

fn main() {
    print_welcome_message();

    let mut loaded_config = true;
    let mut config =
        confy::load::<TwilioConfig>("twilio_cli", "profile").unwrap_or_else(|err| match err {
            _ => {
                eprintln!("Unable to load profile configuration: {}", err);
                TwilioConfig {
                    ..Default::default()
                }
            }
        });

    if config.account_sid.is_empty() | config.auth_token.is_empty() {
        loaded_config = false;
        config = request_credentials();
    } else {
        println!("Twilio profile loaded! {}", config.account_sid);
        println!("");
        println!("");
    }

    let twilio = twilio_rust::Client::new(&config);

    if !loaded_config {
        println!("Checking account...");
        let account = twilio
            .get_account(None)
            .unwrap_or_else(|error| panic!("{}", error));

        println!(
            "✅ Account details good! {} ({} - {})",
            account.friendly_name, account.type_field, account.status
        );

        confy::store("twilio_cli", "profile", &config)
            .unwrap_or_else(|err| eprintln!("Unable to store profile configuration: {}", err));
    }

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

                    let status_options = Status::iter().collect();
                    let status = Select::new("Filter by status?:", status_options).prompt();

                    let mut accounts = twilio
                        .list_accounts(Some(&friendly_name), Some(&status.unwrap()))
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

fn print_welcome_message() {
    println!("");
    println!("");
    println!("");
    println!(
        " _____          _ _ _          ____            _   
|_   _|_      _(_) (_) ___    |  _ \\ _   _ ___| |_ 
  | | \\ \\ /\\ / / | | |/ _ \\   | |_) | | | / __| __|
  | |  \\ V  V /| | | | (_) |  |  _ <| |_| \\__ \\ |_ 
  |_|   \\_/\\_/ |_|_|_|\\___/___|_| \\_\\\\__,_|___/\\__|
                         |_____|                   "
    );
    println!("");
    println!("Welcome to Twilio Rust! I'm here to help you interact with Twilio!");
    println!("");
}
