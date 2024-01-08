use std::process;

use inquire::{validator::Validation, Select, Text};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use twilio_rust::{account::Status, Client};

#[derive(Display, EnumIter, EnumString)]
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
    let options = Action::iter().collect();
    let action_selection = Select::new("Select an action:", options).prompt();

    let action = action_selection.unwrap();

    loop {
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
