mod account;
mod conversation;

use std::{process, str::FromStr};

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use twilio_cli::request_credentials;
use twilio_rust::{self, SubResource, TwilioConfig};

fn main() {
    print_welcome_message();

    let mut loaded_config = false;
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
        config = request_credentials();
    } else {
        if Confirm::new(&format!(
            "Account ({}) found in memory. Use this profile? (Yes / No)",
            config.account_sid
        ))
        .prompt()
        .unwrap()
        {
            loaded_config = true;
        } else {
            config = request_credentials();
        }
    }

    let twilio = twilio_rust::Client::new(&config);

    if !loaded_config {
        println!("Checking account...");
        let account = twilio
            .accounts()
            .get(None)
            .unwrap_or_else(|error| panic!("{}", error));

        println!(
            "✅ Account details good! {} ({} - {})",
            account.friendly_name, account.type_field, account.status
        );

        confy::store("twilio_cli", "profile", &config)
            .unwrap_or_else(|err| eprintln!("Unable to store profile configuration: {}", err));
    }

    loop {
        let mut sub_resource_options: Vec<String> = SubResource::iter()
            .map(|sub_resource| sub_resource.to_string())
            .collect();
        let mut exit_option = vec![String::from("Exit")];
        sub_resource_options.append(&mut exit_option);
        let sub_resource_choice = Select::new("Select a resource:", sub_resource_options)
            .prompt()
            .unwrap();

        // Top level so only 'exit' option.
        if sub_resource_choice == "Exit" {
            process::exit(0);
        }

        let sub_resource = SubResource::from_str(&sub_resource_choice).unwrap();

        match sub_resource {
            twilio_rust::SubResource::Account => account::choose_account_action(&twilio),
            twilio_rust::SubResource::Conversations => {
                conversation::choose_conversation_account(&twilio)
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
