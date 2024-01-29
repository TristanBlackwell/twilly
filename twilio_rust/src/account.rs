/*!

Contains Twilio account related functionality.

*/

use std::{collections::HashMap, fmt};

use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use crate::{Client, TwilioError};

/// Holds account related functions accessible
/// on the client.
pub struct Accounts<'a> {
    pub client: &'a Client,
}

/// Represents a page of accounts from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct AccountPage {
    first_page_uri: String,
    end: u16,
    previous_page_uri: Option<String>,
    accounts: Vec<Account>,
    uri: String,
    page_size: u16,
    start: u16,
    next_page_uri: Option<String>,
    page: u16,
}

/// Details related to a specific account.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub status: Status,
    pub date_updated: String,
    //pub auth_token: String,
    pub friendly_name: String,
    pub owner_account_sid: String,
    pub uri: String,
    pub sid: String,
    pub date_created: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.sid, self.status)
    }
}

/// Possible account statuses.
#[derive(
    AsRefStr, Clone, Display, Debug, EnumIter, EnumString, Serialize, Deserialize, PartialEq,
)]
pub enum Status {
    #[strum(to_string = "Active")]
    #[serde(rename = "active")]
    Active,
    #[strum(to_string = "Suspended")]
    #[serde(rename = "suspended")]
    Suspended,
    #[strum(to_string = "Closed")]
    #[serde(rename = "closed")]
    Closed,
}

impl Default for Status {
    fn default() -> Self {
        Status::Active
    }
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            &Status::Active => "active",
            &Status::Suspended => "suspended",
            &Status::Closed => "closed",
        }
    }
}

impl<'a> Accounts<'a> {
    /// [Gets an Account](https://www.twilio.com/docs/iam/api/account#fetch-an-account-resource)
    ///
    /// Takes in an optional `sid` argument otherwise will default to the current config
    /// account SID.
    pub fn get(&self, sid: Option<&str>) -> Result<Account, TwilioError> {
        let account = self.client.send_request::<Account>(
            Method::GET,
            &format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                sid.unwrap_or_else(|| &self.client.config.account_sid)
            ),
            None,
        );

        account
    }

    /// [Lists Accounts](https://www.twilio.com/docs/iam/api/account#read-multiple-account-resources)
    ///
    /// This will list the account being used for the request and any sub-accounts that match
    /// the provided criteria.
    ///
    /// Accounts will be _eagerly_ paged until all retrieved.
    ///
    /// Takes optional parameters:
    /// - `friendly_name` - Return only accounts matching this friendly name
    /// - `status` - Return only accounts that match this status
    pub fn list(
        &self,
        friendly_name: Option<&str>,
        status: Option<&Status>,
    ) -> Result<Vec<Account>, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(friendly_name) = friendly_name {
            params.insert(String::from("FriendlyName"), friendly_name);
        }

        let status_text = if let Some(status) = status {
            status.to_string()
        } else {
            String::from("")
        };
        if !status_text.is_empty() {
            params.insert(String::from("Status"), &status_text);
        }

        let mut accounts_page = self.client.send_request::<AccountPage>(
            Method::GET,
            "https://api.twilio.com/2010-04-01/Accounts.json?PageSize=5",
            Some(&params),
        )?;

        let mut results: Vec<Account> = accounts_page.accounts;

        while (accounts_page.next_page_uri).is_some() {
            let full_url = format!(
                "https://api.twilio.com{}",
                accounts_page.next_page_uri.unwrap()
            );
            accounts_page =
                self.client
                    .send_request::<AccountPage>(Method::GET, &full_url, None)?;

            results.append(&mut accounts_page.accounts);
        }

        Ok(results)
    }

    /// [Creates a sub-account](https://www.twilio.com/docs/iam/api/account#create-an-account-resource)
    /// under the authenticated Twilio account. Takes in an optional
    /// `friendly_name` argument otherwise defaults to _SubAccount Created at {YYYY-MM-DD HH:MM meridian}_.
    ///
    /// Care should be taken when creating sub-accounts.
    /// - Sub-accounts cannot create other sub-accounts
    /// - Trial accounts can only have a single sub-account beneath it.
    /// See documentation for detail.
    pub fn create(&self, friendly_name: Option<&str>) -> Result<Account, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(friendly_name) = friendly_name {
            params.insert(String::from("FriendlyName"), friendly_name);
        }

        self.client.send_request::<Account>(
            Method::POST,
            "https://api.twilio.com/2010-04-01/Accounts.json",
            Some(&params),
        )
    }

    /// [Updates an account resource](https://www.twilio.com/docs/iam/api/account#update-an-account-resource)
    /// under the authenticated Twilio account.
    ///
    /// Takes the account SID of the account to update and an optional friendly name
    /// and/or status
    pub fn update(
        &self,
        account_sid: &str,
        friendly_name: Option<&str>,
        status: Option<&Status>,
    ) -> Result<Account, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(friendly_name) = friendly_name {
            params.insert(String::from("FriendlyName"), friendly_name);
        }
        let status_text = if let Some(status) = status {
            status.to_string()
        } else {
            String::from("")
        };
        if !status_text.is_empty() {
            params.insert(String::from("Status"), &status_text);
        }
        self.client.send_request::<Account>(
            Method::POST,
            &format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                account_sid
            ),
            Some(&params),
        )
    }
}
