use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display};

use crate::{Client, SubResource, TwilioError};

#[derive(Deserialize)]
#[serde(bound = "T: Deserialize<'de>")]
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

#[derive(AsRefStr, Display)]
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

        let account =
            self.send_request::<Account>(Method::GET, SubResource::Account, Some(&params));

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

        let accounts_first_page =
            self.send_request::<Page<Account>>(Method::GET, SubResource::Account, Some(&params))?;

        let accounts = self.page_until_end(accounts_first_page);
        accounts
    }

    fn page_until_end<'de, Account>(
        &self,
        current_page: Page<Account>,
    ) -> Result<Vec<Account>, TwilioError>
    where
        Account: Deserialize<'de>,
    {
        let mut res: Vec<Account> = current_page.accounts;

        let mut next_page_uri = current_page.next_page_uri;
        while Some(next_page_uri) != None {
            let mut page =
                self.send_request::<Page<Account>>(Method::GET, SubResource::Account, None)?;

            res.append(&mut page.accounts);
            next_page_uri = page.next_page_uri;
        }

        Ok(res)
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

        self.send_request::<Account>(Method::POST, SubResource::Account, Some(&params))
    }
}
