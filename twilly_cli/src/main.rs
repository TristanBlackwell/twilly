mod account;
mod conversation;
mod sync;

use std::{process, str::FromStr};

use inquire::{Confirm, Select};
use strum::IntoEnumIterator;
use twilly::{self, SubResource, TwilioConfig};
use twilly_cli::{prompt_user_selection, request_credentials};

fn main() {
    print_welcome_message();

    let mut loaded_config = false;
    let mut config =
        confy::load::<TwilioConfig>("twilly", "profile").unwrap_or_else(|err| match err {
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

    let twilio = twilly::Client::new(&config);

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

        confy::store("twilly", "profile", &config)
            .unwrap_or_else(|err| eprintln!("Unable to store profile configuration: {}", err));
    }

    loop {
        let mut sub_resource_options: Vec<String> = SubResource::iter()
            .map(|sub_resource| sub_resource.to_string())
            .collect();
        let mut exit_option = vec![String::from("Exit")];
        sub_resource_options.append(&mut exit_option);
        let sub_resource_choice_prompt = Select::new("Select a resource:", sub_resource_options);
        let sub_resource_choice = prompt_user_selection(sub_resource_choice_prompt);

        if sub_resource_choice.is_none() {
            process::exit(0);
        }

        let sub_resource = sub_resource_choice.unwrap();

        // Top level so only 'exit' option.
        if sub_resource == "Exit" {
            process::exit(0);
        }

        let sub_resource = SubResource::from_str(&sub_resource).unwrap();

        match sub_resource {
            twilly::SubResource::Account => account::choose_account_action(&twilio),
            twilly::SubResource::Conversations => conversation::choose_conversation_action(&twilio),
            twilly::SubResource::Sync => sync::choose_sync_resource(&twilio),
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
    println!("Welcome to Twilly! I'm here to help you interact with Twilio!");
    println!("");
}
