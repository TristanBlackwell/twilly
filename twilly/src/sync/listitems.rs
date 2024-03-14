/*!

Contains Twilio Sync List Item related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Represents a page of Sync List Items from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ListItemPage {
    items: Vec<SyncListItem>,
    meta: PageMeta,
}

/// A Sync List Item resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncListItem {
    pub index: u32,
    pub account_sid: String,
    pub service_sid: String,
    pub list_sid: String,
    pub url: String,
    pub data: Value,
    pub date_created: String,
    pub date_updated: String,
    pub date_expires: Option<String>,
    /// Identity of the creator. Uses the identity of the
    /// respective client or defaults to `system` if created via REST.
    pub created_by: String,
    pub revision: String,
}

/// Parameters for creating a Sync List Item
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    data: Value,
    /// How long the List Item should exist before deletion (in seconds).
    ttl: Option<u16>,
    /// How long the *parent* List resource should exist before deletion (in seconds).
    collection_ttl: Option<u16>,
}

#[derive(Serialize)]
pub enum Order {
    Asc,
    Desc,
}

/// See `ListParams`
#[derive(Serialize)]
pub enum Bounds {
    Inclusive,
    Exclusive,
}

/// Arguments for listing Sync Map Items
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct ListParams {
    pub order: Option<Order>,
    // The key of the first Map Item to read.
    pub from: Option<String>,
    /// Whether to include the Map Item described by the `from` parameter. Defaults to inclusive.
    pub bounds: Option<Bounds>,
}

/// Parameters for updating a Sync Map Item
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct UpdateParams {
    #[serde(rename(serialize = "If-Match"))]
    if_match: Option<String>,
    data: Value,
    /// How long the Map Item should exist before deletion (in seconds).
    ttl: Option<u16>,
    /// How long the *parent* Map resource should exist before deletion (in seconds). Can only be used
    /// if the `data` or `ttl` is updated in the same request.
    collection_ttl: Option<u16>,
}

pub struct ListItems<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub list_sid: &'b str,
}

impl<'a, 'b> ListItems<'a, 'b> {
    /// [Creates a Sync List Item](https://www.twilio.com/docs/sync/api/listitem-resource#create-a-listitem-resource)
    ///
    /// Creates a Sync List Item with the provided parameters.
    pub fn create(&self, params: CreateParams) -> Result<SyncListItem, TwilioError> {
        let list_item = self.client.send_request::<SyncListItem, CreateParams>(
            Method::POST,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Lists/{}/Items",
                self.service_sid, self.list_sid
            ),
            Some(&params),
            None,
        );

        list_item
    }

    /// [Lists Sync List Items](https://www.twilio.com/docs/sync/api/listitem-resource#read-multiple-listitem-resources)
    ///
    /// List Sync List Items In the targeted Service and List.
    ///
    /// Targets the Sync Service provided to the `service()` argument, the List provided to the `list()`
    /// argument and lists all List items.
    ///
    /// List items will be _eagerly_ paged until all retrieved.
    pub fn list(&self, params: ListParams) -> Result<Vec<SyncListItem>, TwilioError> {
        let mut list_items_page = self.client.send_request::<ListItemPage, ListParams>(
            Method::GET,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items?PageSize=50",
                self.service_sid, self.list_sid
            ),
            Some(&params),
            None,
        )?;

        let mut results: Vec<SyncListItem> = list_items_page.items;

        while (list_items_page.meta.next_page_url).is_some() {
            list_items_page = self.client.send_request::<ListItemPage, ListParams>(
                Method::GET,
                &list_items_page.meta.next_page_url.unwrap(),
                None,
                None,
            )?;

            results.append(&mut list_items_page.items);
        }

        Ok(results)
    }
}

pub struct ListItem<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub list_sid: &'b str,
    /// Index of the Sync List Item
    pub index: &'b u32,
}

impl<'a, 'b> ListItem<'a, 'b> {
    /// [Gets a Sync List Item](https://www.twilio.com/docs/sync/api/listitem-resource#fetch-a-listitem-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument, the List provided to the `list()`
    /// argument and fetches the item with the index provided to `listitem()`.
    pub fn get(&self) -> Result<SyncListItem, TwilioError> {
        let list_item = self.client.send_request::<SyncListItem, ()>(
            Method::GET,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Lists/{}/Items/{}",
                self.service_sid, self.list_sid, self.index
            ),
            None,
            None,
        );

        list_item
    }

    /// [Update a Sync List Item](https://www.twilio.com/docs/sync/api/listitem-resource#update-a-listitem-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument, the List provided to the `list()`
    /// argument and updates the item with the index provided to `listitem()` with the parameters.
    pub fn update(&self, params: UpdateParams) -> Result<SyncListItem, TwilioError> {
        let headers = HeaderMap::new();

        if let Some(if_match) = params.if_match {
            headers.append("If-Match", if_match);
        }

        let list_item = self.client.send_request::<SyncListItem, UpdateParams>(
            Method::POST,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Lists/{}/Items/{}",
                self.service_sid, self.list_sid, self.index
            ),
            Some(&params),
            Some(headers),
        );

        list_item
    }

    /// [Deletes a Sync List Item](https://www.twilio.com/docs/sync/api/listitem-resource#delete-a-listitem-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument, the List provided to the `list()`
    /// argument and deletes the item with the index provided to `listitem()`.
    pub fn delete(&self) -> Result<(), TwilioError> {
        let list_item = self.client.send_request_and_ignore_response::<()>(
            Method::DELETE,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Lists/{}/Items/{}",
                self.service_sid, self.list_sid, self.index
            ),
            None,
            None,
        );

        list_item
    }
}
