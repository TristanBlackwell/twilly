/*!

Contains Twilio Sync List related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::listitems::{ListItem, ListItems};

/// Represents a page of Sync Lists from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SyncListPage {
    lists: Vec<SyncList>,
    meta: PageMeta,
}

/// A Sync List resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncList {
    pub sid: String,
    pub unique_name: String,
    pub account_sid: String,
    pub service_sid: String,
    pub url: String,
    pub date_created: String,
    pub date_updated: String,
    pub date_expires: Option<String>,
    /// Identity of the creator. Uses the identity of the
    /// respective client or defaults to `system` if created via REST.
    pub created_by: String,
    pub links: Links,
    pub revision: String,
}

/// Resources _linked_ to a Sync List
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub items: String,
    pub permissions: String,
}

impl Default for Links {
    fn default() -> Self {
        Links {
            items: String::from(""),
            permissions: String::from(""),
        }
    }
}

/// Parameters for creating a Sync List
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    unique_name: Option<String>,
    ttl: Option<bool>,
}

/// Parameters for updating a Sync List
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct UpdateParams {
    ttl: Option<bool>,
}

pub struct Lists<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
}

impl<'a, 'b> Lists<'a, 'b> {
    /// [Creates a Sync List resource](https://www.twilio.com/docs/sync/api/list-resource#create-a-list-resource)
    ///
    /// Creates a Sync List resource with the provided parameters.
    pub async fn create(&self, params: CreateParams) -> Result<SyncList, TwilioError> {
        let list = self
            .client
            .send_request::<SyncList, CreateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Lists",
                    &self.service_sid
                ),
                Some(&params),
                None,
            )
            .await;

        list
    }

    /// [Lists Sync Lists](https://www.twilio.com/docs/sync/api/list-resource#read-multiple-list-resources)
    ///
    /// Lists Sync Lists existing on the Twilio account.
    ///
    /// Lists will be _eagerly_ paged until all retrieved.
    pub async fn list(&self) -> Result<Vec<SyncList>, TwilioError> {
        let mut lists_page = self
            .client
            .send_request::<SyncListPage, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Lists?PageSize=50",
                    self.service_sid
                ),
                None,
                None,
            )
            .await?;

        let mut results: Vec<SyncList> = lists_page.lists;

        while (lists_page.meta.next_page_url).is_some() {
            lists_page = self
                .client
                .send_request::<SyncListPage, ()>(
                    Method::GET,
                    &lists_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut lists_page.lists);
        }

        Ok(results)
    }
}

pub struct List<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    /// SID of the Sync List. Can also be it's unique name.
    pub sid: &'b str,
}

impl<'a, 'b> List<'a, 'b> {
    /// [Gets a Sync List](https://www.twilio.com/docs/sync/api/list-resource#fetch-a-list-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and fetches the List
    /// provided to the `list()` argument.
    pub async fn get(&self) -> Result<SyncList, TwilioError> {
        let list = self
            .client
            .send_request::<SyncList, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Lists/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await;

        list
    }

    /// [Update a Sync List](https://www.twilio.com/docs/sync/api/list-resource#update-a-list-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument  and updates the List
    /// provided to the `list()` argument.
    pub async fn update(&self, params: UpdateParams) -> Result<SyncList, TwilioError> {
        let list = self
            .client
            .send_request::<SyncList, UpdateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Lists/{}",
                    self.service_sid, self.sid
                ),
                Some(&params),
                None,
            )
            .await;

        list
    }

    /// [Deletes a Sync List](https://www.twilio.com/docs/sync/api/list-resource#delete-a-list-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and deletes the List
    /// provided to the `list()` argument.
    ///
    /// This will delete any Sync List items underneath this list.
    pub async fn delete(&self) -> Result<(), TwilioError> {
        let list = self
            .client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Lists/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await;

        list
    }

    /// Functions relating to a known Sync List Item.
    ///
    /// Takes in the key of the Sync List Item to perform actions against.
    pub fn listitem(&'a self, index: &'b u32) -> ListItem {
        ListItem {
            client: self.client,
            service_sid: self.service_sid,
            list_sid: self.sid,
            index,
        }
    }

    /// General Sync Map Item functions.
    pub fn listitems(&'a self) -> ListItems {
        ListItems {
            client: self.client,
            service_sid: self.service_sid,
            list_sid: self.sid,
        }
    }
}
