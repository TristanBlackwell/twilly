use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Text};
use serde::{Deserialize, Serialize};

pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
}

impl TwilioConfig {
    pub fn build() -> Result<TwilioConfig, InquireError> {
        println!("Lets start with an account.");

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
                34 => Ok(Validation::Valid),
                _ => Ok(Validation::Invalid(
                    "Your SID should be 34 characters in length".into(),
                )),
            })
            .with_display_mode(PasswordDisplayMode::Masked)
            .with_display_toggle_enabled()
            .without_confirmation()
            .with_help_message("Input is masked. Use Ctrl + R to toggle visibility.")
            .prompt()?;

        Ok(TwilioConfig {
            account_sid,
            auth_token,
        })
    }
}

struct Twilio {
    config: TwilioConfig,
    client: reqwest::blocking::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    sid: String,
    friendly_name: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Error {
    code: u32,
    message: String,
    more_info: String,
    status: u16,
}

impl Twilio {
    fn new(config: TwilioConfig) -> Twilio {
        Twilio {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get_account(&self) -> Result<Account, reqwest::Error> {
        let account: Account = self
            .client
            .get(format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                self.config.account_sid
            ))
            .send()?
            .json()?;

        Ok(account)
    }

    // fn send_request(&self, method: Method, endpoint: &str) {
    // 	let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/{}.json")
    // }
}

pub fn run(config: TwilioConfig) -> Result<(), Error> {
    println!(
        "Generating Twilio Client for account: {}",
        &config.account_sid
    );

    let twilio = Twilio::new(config);

    println!("Checking account...");
    let account = twilio.get_account()?;

    println!("Account = {:#?}", account);

    Ok(())
}
