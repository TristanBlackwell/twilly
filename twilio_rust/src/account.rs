use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{Client, SubResource, TwilioError};

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

impl Client {
    /// [Gets an Account](https://www.twilio.com/docs/iam/api/account#fetch-an-account-resource)
    ///
    /// Takes in an optional `sid` argument otherwise will default to the currently authenticated
    /// account.
    pub fn get_account(&self, sid: Option<&str>) -> Result<Account, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(sid) = sid {
            params.insert(String::from("sid"), sid);
        }

        let account =
            self.send_request::<Account>(Method::GET, SubResource::Account, Some(&params));

        account
    }

    /// [Creates a sub-account](https://www.twilio.com/docs/iam/api/account#create-an-account-resource)
    /// under the authenticated Twilio account. Takes in an optional
    /// `friendly_name` argument otherwise defaults to _SubAccount Created at {YYYY-MM-DD HH:MM meridian}_.
    ///
    /// Care should be taken when creating sub-accounts.
    /// - Sub-accounts cannot create other sub-accounts
    /// - Trial accounts can only have a single sub-account beneath it.
    pub fn create_account(&self, friendly_name: Option<&str>) -> Result<Account, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(friendly_name) = friendly_name {
            params.insert(String::from("friendlyName"), friendly_name);
        }

        self.send_request::<Account>(Method::POST, SubResource::Account, Some(&params))
    }
}
