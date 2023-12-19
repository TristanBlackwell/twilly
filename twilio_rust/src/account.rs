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
    /// Gets the details for the authenticated Twilio account
    pub fn get_account(&self) -> Result<Account, TwilioError> {
        let account = self.send_request::<Account>(Method::GET, SubResource::Account);

        account
    }

    pub fn create_account(&self, friendlyName: Option<&str>) -> Result<Account, TwilioError> {
        let created_account = self.send_request::<Account>(Method::POST, SubResource::Account);
    }
}
