mod account;

use std::fmt;

use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Account SID & auth token pair required for
/// authenticating requests to Twilio.
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

impl Client {
    pub fn new(config: TwilioConfig) -> Client {
        Client {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Dispatches a request to Twilio and handles parsing the response.
    ///
    /// Will return a result of either the resource type or one of the
    /// possible errors ([`Error`]).
    ///
    /// This method may throw on failed network requests.
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
