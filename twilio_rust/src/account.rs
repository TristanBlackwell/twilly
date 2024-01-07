use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::{Client, TwilioError};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Page<T> {
    first_page_uri: String,
    end: u16,
    previous_page_uri: Option<String>,
    accounts: Vec<T>,
    uri: String,
    page_size: u16,
    start: u16,
    next_page_uri: Option<String>,
    page: u16,
}

/// Details related to a specific account.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub status: String,
    pub date_updated: String,
    pub auth_token: String,
    pub friendly_name: String,
    pub owner_account_sid: String,
    pub uri: String,
    pub sid: String,
    pub date_created: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(AsRefStr, Display, EnumIter)]
pub enum Status {
    Active,
    Closed,
    Suspended,
}

impl Client {
    /// [Gets an Account](https://www.twilio.com/docs/iam/api/account#fetch-an-account-resource)
    ///
    /// Takes in an optional `sid` argument otherwise will default to the current config
    /// account SID.
    pub fn get_account(&self, sid: Option<&str>) -> Result<Account, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(sid) = sid {
            params.insert(String::from("sid"), sid);
        }

        let account = self.send_request::<Account>(
            Method::GET,
            &format!(
                "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                self.config.account_sid
            ),
            Some(&params),
        );

        account
    }

    /// [Lists Accounts](https://www.twilio.com/docs/iam/api/account#read-multiple-account-resources)
    ///
    /// This will list the account being used for the request and any sub-accounts that match
    /// the provided criteria
    ///
    /// Takes optional parameters:
    /// - `friendly_name` - Return only accounts matching this friendly name
    /// - `status` - Return only accounts that match this status
    pub fn list_accounts(
        &self,
        friendly_name: Option<&str>,
        status: Option<&Status>,
    ) -> Result<Vec<Account>, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(friendly_name) = friendly_name {
            params.insert(String::from("FriendlyName"), friendly_name);
        }
        if let Some(status) = status {
            params.insert(String::from("Status"), status.as_ref());
        }

        let mut accounts_page = self.send_request::<Page<Account>>(
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
            accounts_page = self.send_request::<Page<Account>>(Method::GET, &full_url, None)?;

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
    pub fn create_account(&self, friendly_name: Option<&str>) -> Result<Account, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(friendly_name) = friendly_name {
            params.insert(String::from("friendlyName"), friendly_name);
        }

        self.send_request::<Account>(
            Method::POST,
            "https://api.twilio.com/2010-04-01/Accounts.json",
            Some(&params),
        )
    }
}
