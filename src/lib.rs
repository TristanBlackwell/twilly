use std::fmt;

use inquire::{validator::Validation, InquireError, Password, PasswordDisplayMode, Text};
use reqwest::Method;
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

pub struct TwilioError {
    kind: ErrorKind,
}

impl fmt::Display for TwilioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind.as_str())
    }
}

/// A list of possible errors from the Twilio client.
pub enum ErrorKind {
    /// Network related error during the request.
    NetworkError,
    /// Twilio returned error
    TwilioError(TwilioApiError),
    /// Unable to parse request or response body
    ParsingError,
}

impl ErrorKind {
    fn as_str(&self) -> String {
        match self {
            ErrorKind::NetworkError => "Network error reaching Twilio.".to_string(),
            ErrorKind::ParsingError => "Unable to parse response.".to_string(),
            ErrorKind::TwilioError(error) => {
                format!("Error response from Twilio: {}", &error)
            }
        }
    }
}

/// Twilio error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct TwilioApiError {
    /// Twilio specific error code
    code: u32,
    /// Detail of the error
    message: String,
    /// Where to find more info on the error
    more_info: String,
    /// HTTP status code
    status: u16,
}

impl fmt::Display for TwilioApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} from Twilio. ({}) {}. For more info see: {}",
            self.status, self.code, self.message, self.more_info
        )
    }
}

pub enum SubResource {
    Account,
    Sync,
}

impl fmt::Display for SubResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Details related to a specific account.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Account {
    status: String,
    date_updated: String,
    auth_token: String,
    friendly_name: String,
    owner_account_sid: String,
    uri: String,
    sid: String,
    date_created: String,
    #[serde(rename = "type")]
    type_field: String,
}

impl Twilio {
    fn new(config: TwilioConfig) -> Twilio {
        Twilio {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get_account(&self) -> Result<Account, TwilioError> {
        let account = self.send_request::<Account>(Method::GET, SubResource::Account);

        account
    }

    /// Dispatches a request to Twilio and handles parsing the response.
    ///
    /// Will return a result of either the resource type or one of the
    /// possible errors ([`Error`])
    fn send_request<T>(&self, method: Method, endpoint: SubResource) -> Result<T, TwilioError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = match endpoint {
            SubResource::Account => format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                self.config.account_sid
            ),
            _ => format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}/{}.json",
                self.config.account_sid, endpoint
            ),
        };

        let response = self
            .client
            .request(method, url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .unwrap();

        match reqwest::StatusCode::OK {
            reqwest::StatusCode::OK => response.json::<T>().map_err(|_| TwilioError {
                kind: ErrorKind::ParsingError,
            }),
            _ => match response.json::<TwilioApiError>() {
                Ok(parsed) => Err(TwilioError {
                    kind: ErrorKind::TwilioError(parsed),
                }),
                Err(_) => panic!("Err!"),
            },
        }
    }
}

pub fn run(config: TwilioConfig) -> Result<(), TwilioApiError> {
    println!(
        "Generating Twilio Client for account: {}",
        &config.account_sid
    );

    let twilio = Twilio::new(config);

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

    Ok(())
}
