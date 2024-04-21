/*!

Contains Twilio account related functionality.

*/

use std::fmt;

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

/// Possible Account statuses.
#[derive(
    AsRefStr, Clone, Display, Debug, EnumIter, EnumString, Serialize, Deserialize, PartialEq,
)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// A live account.
    #[strum(to_string = "Active")]
    Active,
    /// The account has been temporarily suspended by it's parent.
    #[strum(to_string = "Suspended")]
    Suspended,
    /// A closed account. This is permanent and cannot be re-opened.
    #[strum(to_string = "Closed")]
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
            Status::Active => "active",
            Status::Suspended => "suspended",
            Status::Closed => "closed",
        }
    }
}

/// Possible filters when listing Accounts via the Twilio API
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct ListOrUpdateParams {
    pub friendly_name: Option<String>,
    pub status: Option<Status>,
}

/// Possible options when creating an Account via the Twilio API
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    pub friendly_name: Option<String>,
}

impl<'a> Accounts<'a> {
    /// [Gets an Account](https://www.twilio.com/docs/iam/api/account#fetch-an-account-resource)
    ///
    /// Takes in an optional `sid` argument otherwise will default to the current config
    /// account SID.
    pub async fn get(&self, sid: Option<&str>) -> Result<Account, TwilioError> {
        self.client
            .send_request::<Account, ()>(
                Method::GET,
                &format!(
                    "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                    sid.unwrap_or_else(|| &self.client.config.account_sid)
                ),
                None,
                None,
            )
            .await
    }

    /// [Lists Accounts](https://www.twilio.com/docs/iam/api/account#read-multiple-account-resources)
    ///
    /// This will list all accounts that match the provided criteria.
    /// This scans all subaccounts and can also include the account making the request.
    ///
    /// Accounts will be _eagerly_ paged until all retrieved.
    ///
    /// Takes optional parameters:
    /// - `friendly_name` - Return only accounts matching this friendly name
    /// - `status` - Return only accounts that match this status
    pub async fn list(
        &self,
        friendly_name: Option<&str>,
        status: Option<&Status>,
    ) -> Result<Vec<Account>, TwilioError> {
        let params = ListOrUpdateParams {
            friendly_name: friendly_name.map(|friendly_name| friendly_name.to_string()),
            status: status.cloned(),
        };

        let mut accounts_page = self
            .client
            .send_request::<AccountPage, ListOrUpdateParams>(
                Method::GET,
                "https://api.twilio.com/2010-04-01/Accounts.json?PageSize=5",
                Some(&params),
                None,
            )
            .await?;

        let mut results: Vec<Account> = accounts_page.accounts;

        while (accounts_page.next_page_uri).is_some() {
            let full_url = format!(
                "https://api.twilio.com{}",
                accounts_page.next_page_uri.unwrap()
            );
            accounts_page = self
                .client
                .send_request::<AccountPage, ()>(Method::GET, &full_url, None, None)
                .await?;

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
    ///
    /// See documentation for detail.
    pub async fn create(&self, friendly_name: Option<&str>) -> Result<Account, TwilioError> {
        let params = CreateParams {
            friendly_name: friendly_name.map(|friendly_name| friendly_name.to_string()),
        };

        self.client
            .send_request::<Account, CreateParams>(
                Method::POST,
                "https://api.twilio.com/2010-04-01/Accounts.json",
                Some(&params),
                None,
            )
            .await
    }

    /// [Updates an account resource](https://www.twilio.com/docs/iam/api/account#update-an-account-resource)
    /// under the authenticated Twilio account.
    ///
    /// - `account_sid` -  the account SID of the account to update.
    ///
    /// Takes optional parameters:
    /// - `friendly_name` - Update the friendly name to the provided value
    /// - `status` - Change the account status
    pub async fn update(
        &self,
        account_sid: &str,
        friendly_name: Option<&str>,
        status: Option<&Status>,
    ) -> Result<Account, TwilioError> {
        let opts = ListOrUpdateParams {
            friendly_name: friendly_name.map(|friendly_name| friendly_name.to_string()),
            status: status.cloned(),
        };

        self.client
            .send_request::<Account, ListOrUpdateParams>(
                Method::POST,
                &format!(
                    "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                    account_sid
                ),
                Some(&opts),
                None,
            )
            .await
    }
}
