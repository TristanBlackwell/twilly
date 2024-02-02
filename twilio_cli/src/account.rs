use std::{process, str::FromStr};

use inquire::{validator::Validation, Confirm, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilio_cli::{
    get_action_choice_from_user, get_filter_choice_from_user, prompt_user, prompt_user_selection,
    ActionChoice, FilterChoice,
};
use twilio_rust::{
    account::{Account, Status},
    Client,
};

#[derive(Debug, Clone, Display, EnumIter, EnumString)]
pub enum Action {
    #[strum(to_string = "Get account")]
    GetAccount,
    #[strum(to_string = "List accounts")]
    ListAccounts,
    #[strum(to_string = "Create account")]
    CreateAccount,
    Back,
    Exit,
}

pub fn choose_account_action(twilio: &Client) {
    let options: Vec<Action> = Action::iter().collect();

    loop {
        let action_selection_prompt = Select::new("Select an action:", options.clone());
        let action_selection = prompt_user_selection(action_selection_prompt);
        if action_selection.is_none() {
            break;
        }

        let action = action_selection.unwrap();
        match action {
            Action::GetAccount => {
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
                let account_sid = prompt_user(account_sid_prompt);

                if account_sid.is_some() {
                    let account = twilio
                        .accounts()
                        .get(Some(&account_sid.unwrap()))
                        .unwrap_or_else(|error| panic!("{}", error));
                    println!("{:#?}", account);
                    println!();
                }
            }
            Action::CreateAccount => {
                let friendly_name_prompt = Text::new("Enter a friendly name (empty for default):");
                let friendly_name = prompt_user(friendly_name_prompt);

                if friendly_name.is_some() {
                    println!("Creating account...");
                    let account = twilio
                        .accounts()
                        .create(Some(&friendly_name.unwrap()))
                        .unwrap_or_else(|error| panic!("{}", error));
                    println!(
                        "Account created: {} ({})",
                        account.friendly_name, account.sid
                    );
                }
            }
            Action::ListAccounts => {
                let friendly_name_prompt = Text::new("Search by friendly name? (empty for none):");
                let friendly_name_opt = prompt_user(friendly_name_prompt);

                if friendly_name_opt.is_some() {
                    let friendly_name = friendly_name_opt.unwrap();
                    let status_choice_opt = get_filter_choice_from_user(
                        Status::iter().map(|status| status.to_string()).collect(),
                        "Filter by status: ",
                    );

                    if status_choice_opt.is_some() {
                        let status = match status_choice_opt.unwrap() {
                            FilterChoice::Any => None,
                            FilterChoice::Other(choice) => Some(Status::from_str(&choice).unwrap()),
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
                            let account_action_choice = get_action_choice_from_user(
                                modifiable_accounts
                                    .iter()
                                    .map(|ac| {
                                        format!("({}) {} - {}", ac.sid, ac.friendly_name, ac.status)
                                    })
                                    .collect::<Vec<String>>(),
                                "Accounts: ",
                            );

                            let selected_account = match account_action_choice {
                                Some(account_action) => match account_action {
                                    ActionChoice::Back => break,
                                    ActionChoice::Exit => process::exit(0),
                                    ActionChoice::Other(choice) => {
                                        accounts.iter().find(|ac| ac.sid == choice[1..35]).unwrap()
                                    }
                                },
                                None => break,
                            };

                            match selected_account.status.as_str() {
                                "active" => {
                                    let selected_account_action = get_action_choice_from_user(
                                        vec![
                                            "Change name".into(),
                                            "Suspend".into(),
                                            "Close".into(),
                                        ],
                                        "Select an action: ",
                                    );

                                    match selected_account_action {
                                        Some(selected_account_action) => {
                                            match selected_account_action {
                                                ActionChoice::Back => break,
                                                ActionChoice::Exit => process::exit(0),
                                                ActionChoice::Other(choice) => {
                                                    match choice.as_str() {
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
                                                        _ => {
                                                            println!("Unknown action '{}'", choice)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        None => break,
                                    }
                                }
                                "suspended" => {
                                    let selected_account_action = get_action_choice_from_user(
                                        vec!["Change name".into(), "Activate".into()],
                                        "Select an action: ",
                                    );

                                    match selected_account_action {
                                        Some(selected_account_action) => {
                                            match selected_account_action {
                                                ActionChoice::Back => break,
                                                ActionChoice::Exit => process::exit(0),
                                                ActionChoice::Other(choice) => {
                                                    match choice.as_str() {
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
                                                        _ => {
                                                            println!("Unknown action '{}'", choice)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        None => break,
                                    };
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
                }
            }
            Action::Back => break,
            Action::Exit => process::exit(0),
        }
    }
}

fn change_account_name(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    let friendly_name_prompt =
        Text::new("Provide a name:").with_validator(|val: &str| match val.len() > 0 {
            true => Ok(Validation::Valid),
            false => Ok(Validation::Invalid("Enter at least one character".into())),
        });
    let friendly_name_opt = prompt_user(friendly_name_prompt);

    if friendly_name_opt.is_some() {
        let friendly_name = friendly_name_opt.unwrap();
        println!("Updating account...");
        let updated_account = twilio
            .accounts()
            .update(account_sid, Some(&friendly_name), None)
            .unwrap_or_else(|error| panic!("{}", error));

        println!("{:#?}", updated_account);
        println!("");

        for acc in accounts {
            if acc.sid == account_sid {
                acc.friendly_name = friendly_name.clone();
            }
        }
    }
}

fn activate_account(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    let confirmation_prompt =
        Confirm::new("Are you sure you wish to activate this account? (Yes / No)");
    let confirmation = prompt_user(confirmation_prompt);
    if confirmation.is_some() && confirmation.unwrap() == true {
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
        return;
    }

    println!("Operation canceled. No changes were made.");
}

fn suspend_account(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    let confirmation_prompt =
        Confirm::new("Are you sure you wish to suspend this account? Any activity will be disabled until the account is re-activated. (Yes / No)");
    let confirmation = prompt_user(confirmation_prompt);
    if confirmation.is_some() && confirmation.unwrap() == true {
        println!("Suspending account...");
        let res = twilio
            .accounts()
            .update(account_sid, None, Some(&Status::Suspended))
            .unwrap_or_else(|error| panic!("{}", error));

        println!("{}", res);
        println!("Account suspended.");
        for acc in accounts {
            if acc.sid == account_sid {
                acc.status = Status::Suspended;
            }
        }
        return;
    }

    println!("Operation canceled. No changes were made.");
}

fn close_account(twilio: &Client, account_sid: &str, accounts: &mut Vec<Account>) {
    let confirmation_prompt =
        Confirm::new("Are you sure you wish to Close this account? Activity will be disabled and this action cannot be reversed. (Yes / No)");
    let confirmation = prompt_user(confirmation_prompt);
    if confirmation.is_some() && confirmation.unwrap() == true {
        println!("Closing account...");
        twilio
            .accounts()
            .update(account_sid, None, Some(&Status::Suspended))
            .unwrap_or_else(|error| panic!("{}", error));

        println!("Account closed. This account will still be visible in the console for 30 days.");
        for acc in accounts {
            if acc.sid == account_sid {
                acc.status = Status::Closed;
            }
        }
        return;
    }

    println!("Operation canceled. No changes were made.");
}
