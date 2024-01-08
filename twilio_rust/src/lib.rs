pub mod account;

use std::{collections::HashMap, fmt};

use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

/// Account SID & auth token pair required for
/// authenticating requests to Twilio.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
}

impl TwilioConfig {
    pub fn build(account_sid: String, auth_token: String) -> TwilioConfig {
        if !account_sid.starts_with("AC") {
            panic!("Account SID must start with AC");
        } else if account_sid.len() != 34 {
            panic!(
                "Account SID should be 34 characters in length. Was {}",
                account_sid.len()
            )
        }

        if auth_token.len() != 32 {
            panic!(
                "Auth token should be 32 characters in length. Was {}",
                auth_token.len()
            )
        }

        TwilioConfig {
            account_sid,
            auth_token,
        }
    }
}

/// The Twilio client used for interaction with
/// Twilio's API
pub struct Client {
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
    NetworkError(reqwest::Error),
    /// Twilio returned error
    TwilioError(TwilioApiError),
    /// Unable to parse request or response body
    ParsingError(reqwest::Error),
}

impl ErrorKind {
    fn as_str(&self) -> String {
        match self {
            ErrorKind::NetworkError(error) => format!("Network error reaching Twilio: {}", &error),
            ErrorKind::ParsingError(error) => format!("Unable to parse response: {}", &error),
            ErrorKind::TwilioError(error) => {
                format!("Error: {}", &error)
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

#[derive(Display, EnumIter, EnumString, PartialEq)]
pub enum SubResource {
    Account,
    Sync,
}

impl Client {
    /// Create a Twilio client ready to send requests.
    pub fn new(config: &TwilioConfig) -> Client {
        Client {
            config: config.clone(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Dispatches a request to Twilio and handles parsing the response.
    ///
    /// Will return a result of either the resource type or one of the
    /// possible errors ([`Error`]).
    fn send_request<T>(
        &self,
        method: Method,
        url: &str,
        params: Option<&HashMap<String, &str>>,
    ) -> Result<T, TwilioError>
    where
        T: serde::de::DeserializeOwned,
    {
        let response_result = match method {
            Method::GET => self
                .client
                .request(method, url)
                .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
                .query(&params)
                .send(),
            _ => self
                .client
                .request(method, url)
                .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
                .form(&params)
                .send(),
        };

        let response = match response_result {
            Ok(res) => res,
            Err(error) => {
                return Err(TwilioError {
                    kind: ErrorKind::NetworkError(error),
                })
            }
        };

        match response.status() {
            reqwest::StatusCode::OK => response.json::<T>().map_err(|error| TwilioError {
                kind: ErrorKind::ParsingError(error),
            }),
            _ => {
                let parsed_twilio_error = response.json::<TwilioApiError>();

                match parsed_twilio_error {
                    Ok(twilio_error) => Err(TwilioError {
                        kind: ErrorKind::TwilioError(twilio_error),
                    }),
                    Err(error) => Err(TwilioError {
                        kind: ErrorKind::ParsingError(error),
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Account SID must start with AC")]
    fn account_sid_regex() {
        TwilioConfig::build(String::from("ThisisnotanaccountSID"), String::from("1234"));
    }

    #[test]
    #[should_panic(expected = "Account SID should be 34 characters in length. Was 23")]
    fn account_sid_len() {
        TwilioConfig::build(
            String::from("ACThisisnotanaccountSID"),
            String::from("1234"),
        );
    }

    #[test]
    #[should_panic(expected = "Auth token should be 32 characters in length. Was 20")]
    fn auth_token_len() {
        TwilioConfig::build(
            String::from("AC11111111111111111111111111111111"),
            String::from("11111111111111111111"),
        );
    }

    #[test]
    fn config_on_good_credentials() {
        let account_sid = String::from("AC11111111111111111111111111111111");
        let auth_token = String::from("11111111111111111111111111111111");
        let config = TwilioConfig::build(account_sid.clone(), auth_token.clone());

        assert_eq!(account_sid, config.account_sid);
        assert_eq!(auth_token, config.auth_token);
    }
}
