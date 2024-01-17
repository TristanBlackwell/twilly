use std::{process, str::FromStr};

use inquire::{validator::Validation, Confirm, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilio_rust::{
    account::{Account, Status},
    Client,
};

#[derive(Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(serialize = "Get account")]
    GetAccount,
    #[strum(serialize = "List accounts")]
    ListAccounts,
    #[strum(serialize = "Create account")]
    CreateAccount,
    Back,
    Exit,
}

pub fn choose_account_action(twilio: &Client) {
    let options: Vec<Action> = Action::iter().collect();

    loop {
        let action_selection = Select::new("Select an action:", options.clone()).prompt();
        let action = action_selection.unwrap();
        match action {
            Action::GetAccount => {
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
                    .unwrap();
                let account = twilio
                    .accounts()
                    .get(Some(&account_sid))
                    .unwrap_or_else(|error| panic!("{}", error));
                println!("{:?}", account);
            }
            Action::CreateAccount => {
                let friendly_name = Text::new("Enter a friendly name (empty for default):")
                    .prompt()
                    .unwrap();

                println!("Creating account...");
                let account = twilio
                    .accounts()
                    .create(Some(&friendly_name))
                    .unwrap_or_else(|error| panic!("{}", error));
                println!(
                    "Account created: {} ({})",
                    account.friendly_name, account.sid
                );
            }
            Action::ListAccounts => {
                let friendly_name = Text::new("Search by friendly name? (empty for none):")
                    .prompt()
                    .unwrap();

                let mut status_options: Vec<String> =
                    Status::iter().map(|status| status.to_string()).collect();
                status_options.insert(0, String::from("Any"));
                let status_choice = Select::new("Filter by status?:", status_options)
                    .prompt()
                    .unwrap();

                let status = if status_choice.as_str() == "Any" {
                    None
                } else {
                    Some(Status::from_str(&status_choice).unwrap())
                };

                println!("Retrieving accounts...");
                let mut accounts = twilio
                    .accounts()
                    .list(Some(&friendly_name), status.as_ref())
                    .unwrap_or_else(|error| panic!("{}", error));

                // The accounts we can perform on the account we are currently using are limited.
                // Remove from the list.
                accounts.retain(|ac| ac.sid != twilio.config.account_sid);

                if accounts.len() == 0 {
                    println!("No accounts found.");
                    break;
                }

                println!("Found {} accounts.", accounts.len());

                let mut modifiable_accounts = accounts.clone();

                loop {
                    let mut account_options = modifiable_accounts
                        .iter()
                        .map(|ac| format!("({}) {} - {}", ac.sid, ac.friendly_name, ac.status))
                        .collect::<Vec<String>>();
                    let mut back_and_exit_options =
                        vec![String::from("Back"), String::from("Exit")];
                    account_options.append(&mut back_and_exit_options);
                    let selected_option =
                        Select::new("Accounts:", account_options).prompt().unwrap();

                    if selected_option == "Back" {
                        break;
                    } else if selected_option == "Exit" {
                        process::exit(0);
                    }

                    let selected_account = accounts
                        .iter()
                        .find(|ac| ac.sid == selected_option[1..35])
                        .unwrap();

                    match selected_account.status.as_str() {
                        "active" => {
                            let account_action = Select::new(
                                "Select an action:",
                                vec!["Change name", "Suspend", "Close"],
                            )
                            .prompt()
                            .unwrap();

                            match account_action {
                                "Change name" => {
                                    change_account_name(
                                        twilio,
                                        &selected_account.sid,
                                        &mut modifiable_accounts,
                                    );
                                }
                                "Suspend" => {
                                    suspend_account(
                                        twilio,
                                        &selected_account.sid,
                                        &mut modifiable_accounts,
                                    );
                                }
                                "Close" => {
                                    close_account(
                                        twilio,
                                        &selected_account.sid,
                                        &mut modifiable_accounts,
                                    );
                                }
                                _ => println!("Unknown action '{}'", account_action),
                            }
                        }
                        "suspended" => {
                            let account_action =
                                Select::new("Select an action:", vec!["Change name", "Activate"])
                                    .prompt()
                                    .unwrap();

                            match account_action {
                                "Change name" => change_account_name(
                                    twilio,
                                    &selected_account.sid,
                                    &mut modifiable_accounts,
                                ),
                                "Activate" => activate_account(
                                    twilio,
                                    &selected_account.sid,
                                    &mut modifiable_accounts,
                                ),
                                _ => println!("Unknown action '{}'", account_action),
                            }
                        }
                        "closed" => {
                            println!(
                                "{} is a closed account and can no longer be used.",
                                selected_account.sid
                            );
                        }
                        _ => {
                            println!("Unknown account type '{}'", selected_account.status);
                        }
                    }
                }
            }
            Action::Back => break,
            Action::Exit => process::exit(0),
        }
    }
}

fn change_account_name(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    let friendly_name = Text::new("Provide a name:")
        .with_validator(|val: &str| match val.len() > 0 {
            true => Ok(Validation::Valid),
            false => Ok(Validation::Invalid("Enter at least one character".into())),
        })
        .prompt()
        .unwrap();

    println!("Updating account...");
    let updated_account = twilio
        .accounts()
        .update(account_sid, Some(&friendly_name), None)
        .unwrap_or_else(|error| panic!("{}", error));

    println!("{:?}", updated_account);

    for acc in accounts {
        if acc.sid == account_sid {
            acc.friendly_name = friendly_name.clone();
        }
    }
}

fn activate_account(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    if Confirm::new("Are you sure you wish to activate this account? (Yes / No)")
        .prompt()
        .unwrap()
    {
        println!("Activating account...");
        twilio
            .accounts()
            .update(account_sid, None, Some(&Status::Suspended))
            .unwrap_or_else(|error| panic!("{}", error));

        println!("Account activated.");

        for acc in accounts {
            if acc.sid == account_sid {
                acc.status = Status::Active;
            }
        }
    } else {
        println!("Operation canceled. No changes were made.");
    }
}

fn suspend_account(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    if Confirm::new("Are you sure you wish to suspend this account? Any activity will be disabled until the account is re-activated. (Yes / No)").prompt().unwrap() {
		println!("Suspending account...");
		let res = twilio
			.accounts().update(
				account_sid,
				None,
				Some(&Status::Suspended),
			)
			.unwrap_or_else(|error| panic!("{}", error));

		println!("{}", res);
		println!("Account suspended.");
		for acc in accounts {
            if acc.sid == account_sid {
                acc.status = Status::Suspended;
            }
        }
	} else {
		println!("Operation canceled. No changes were made.");
	}
}

fn close_account(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    if Confirm::new("Are you sure you wish to Close this account? Activity will be disabled and this action cannot be reversed. (Yes / No)").prompt().unwrap() {
		println!("Closing account...");
		twilio
			.accounts().update(
				account_sid,
				None,
				Some(&Status::Suspended),
			)
			.unwrap_or_else(|error| panic!("{}", error));

		println!("Account closed. This account will still be visible in the console for 30 days.");
		for acc in accounts {
            if acc.sid == account_sid {
                acc.status = Status::Closed;
            }
        }
	} else {
		println!("Operation canceled. No changes were made.");
	}
}
