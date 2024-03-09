/*! This crate is an *synchronous* implementation of the Twilio API in Rust built
upon Reqwest and Serde.

Coverage is partial yet provides an idiomatic usage pattern currently covering:

- Accounts
- Conversations

This crate has been developed alongside the `twilly-cli crate which provides an
enhanced Twilio CLI experience.

# Example

Interaction is done via a Twilio client that can be created via the constructor. The config
parameter is a `TwilioConfig` struct of an account SID & auth token pair.

```
let twilio = twilly::Client::new(&config);
```

To retrieve accounts from the client:

```
twilio.accounts().list(Some(&friendly_name), None);
```

To delete a conversation:

```
twilio.conversations().delete(&conversation_sid);
```

*/

pub mod account;
pub mod conversation;
pub mod sync;

use std::fmt::{self};

use account::Accounts;
use conversation::Conversations;
use reqwest::{blocking::Response, Method};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use sync::Sync;

/// Account SID & auth token pair required for
/// authenticating requests to Twilio.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct TwilioConfig {
    /// Twilio account SID, begins with AC...
    pub account_sid: String,
    /// Twilio account auth token
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
/// Twilio's API.
pub struct Client {
    pub config: TwilioConfig,
    client: reqwest::blocking::Client,
}

/// Crate error wrapping containing a `kind` used
/// to differentiate errors.
#[derive(Debug)]
pub struct TwilioError {
    pub kind: ErrorKind,
}

impl fmt::Display for TwilioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind.as_str())
    }
}

/// A list of possible errors from the Twilio client.
#[derive(Debug)]
pub enum ErrorKind {
    /// Validation error related to incoming arguments.
    ValidationError(String),
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
            ErrorKind::ValidationError(error) => {
                format!("Validation error for provided arguments: {}", error)
            }
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
    pub code: u32,
    /// Detail of the error
    pub message: String,
    /// Where to find more info on the error
    pub more_info: String,
    /// HTTP status code
    pub status: u16,
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

/// Holds the page information from the API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct PageMeta {
    page: u16,
    page_size: u16,
    first_page_url: String,
    previous_page_url: Option<String>,
    next_page_url: Option<String>,
    key: String,
}

/// Available Twilio resources to access.
#[derive(Display, EnumIter, EnumString, PartialEq)]
pub enum SubResource {
    Account,
    Conversations,
    Sync,
}

impl Client {
    /// Create a Twilio client ready to send requests based on the
    /// provided config.
    pub fn new(config: &TwilioConfig) -> Self {
        Self {
            config: config.clone(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Dispatches a request to Twilio and handles parsing the response.
    ///
    /// The function takes two generics `T` and `U`. `T` is the expected response
    /// body and `U` is the parameters structre.
    ///
    /// If the method allows for a request body then `params` is sent as
    /// x-www-form-urlencoded otherwise `params` are attached as query
    /// string parameters.
    ///
    /// Will return a result of either the resource type or one of the
    /// possible errors.
    fn send_request<T, U>(
        &self,
        method: Method,
        url: &str,
        params: Option<&U>,
    ) -> Result<T, TwilioError>
    where
        T: serde::de::DeserializeOwned,
        U: Serialize + ?Sized,
    {
        let response = self.send_http_request(method, url, params)?;

        match response.status().is_success() {
            true => response.json::<T>().map_err(|error| TwilioError {
                kind: ErrorKind::ParsingError(error),
            }),
            false => {
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

    /// Dispatches a request to Twilio ignoring the response returned. This is generally
    /// for mutating where either the response is irrelevant or there is nothing returned.
    ///
    /// Params and result follow the same behaviour as `send_request`.
    fn send_request_and_ignore_response<T>(
        &self,
        method: Method,
        url: &str,
        params: Option<&T>,
    ) -> Result<(), TwilioError>
    where
        T: Serialize + ?Sized,
    {
        let response = self.send_http_request(method, url, params)?;

        match response.status().is_success() {
            true => Ok(()),
            false => {
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

    // @INTERNAL
    // Helper function for `send_request`. Not designed to be used independently.
    fn send_http_request<T>(
        &self,
        method: Method,
        url: &str,
        params: Option<&T>,
    ) -> Result<Response, TwilioError>
    where
        T: Serialize + ?Sized,
    {
        match method {
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
        }
        .map_err(|error| TwilioError {
            kind: ErrorKind::NetworkError(error),
        })
    }

    /// Account related functions.
    pub fn accounts<'a>(&'a self) -> Accounts {
        Accounts { client: self }
    }

    /// Conversation related functions.
    pub fn conversations<'a>(&'a self) -> Conversations {
        Conversations { client: self }
    }

    /// Sync related functions.
    pub fn sync<'a>(&'a self) -> Sync {
        Sync { client: self }
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
