/*!

Contains Twilio Sync Map Item related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Represents a page of Sync Map Items from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct MapItemPage {
    items: Vec<SyncMapItem>,
    meta: PageMeta,
}

/// A Sync Map Item resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMapItem {
    pub key: String,
    pub account_sid: String,
    pub service_sid: String,
    pub map_sid: String,
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

/// Arguments for creating a Sync Map Item
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    key: String,
    data: Value,
    /// How long the Map Item should exist before deletion (in seconds).
    ttl: Option<u16>,
    /// How long the *parent* Map resource should exist before deletion (in seconds).
    collection_ttl: Option<u16>,
}

#[derive(Serialize)]
pub enum Order {
    Asc,
    Desc,
}

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
    order: Option<Order>,
    // The key of the first Map Item to read.
    from: Option<String>,
    /// Whether to include the Map Item described by the `from` parameter. Defaults to inclusive.
    bounds: Option<Bounds>,
}

/// Arguments for updating a Sync Map Item
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

pub struct MapItems<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub map_sid: &'b str,
}

impl<'a, 'b> MapItems<'a, 'b> {
    /// [Creates a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#create-a-mapitem-resource)
    ///
    /// Creates a Sync Map Item with the provided parameters.
    pub fn create(&self, params: CreateParams) -> Result<SyncMapItem, TwilioError> {
        let map_item = self.client.send_request::<SyncMapItem, CreateParams>(
            Method::POST,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items",
                self.service_sid, self.map_sid
            ),
            Some(&params),
        );

        map_item
    }

    /// [Lists Sync Map Items](https://www.twilio.com/docs/sync/api/map-item-resource#read-all-mapitem-resources)
    ///
    /// This will list Sync Map Items In the targeted Service and Map.
    ///
    /// Map items will be _eagerly_ paged until all retrieved.
    pub fn list(&self, params: ListParams) -> Result<Vec<SyncMapItem>, TwilioError> {
        let mut map_items_page = self.client.send_request::<MapItemPage, ListParams>(
            Method::GET,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items?PageSize=50",
                self.service_sid, self.map_sid
            ),
            Some(&params),
        )?;

        let mut results: Vec<SyncMapItem> = map_items_page.items;

        while (map_items_page.meta.next_page_url).is_some() {
            map_items_page = self.client.send_request::<MapItemPage, ListParams>(
                Method::GET,
                &map_items_page.meta.next_page_url.unwrap(),
                None,
            )?;

            results.append(&mut map_items_page.items);
        }

        Ok(results)
    }
}

pub struct MapItem<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub map_sid: &'b str,
    /// Key of the Sync Map Item
    pub key: &'b str,
}

impl<'a, 'b> MapItem<'a, 'b> {
    /// [Gets a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#fetch-a-mapitem-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and the Map provided to the `Map` argument
    /// and fetches the key provided
    pub fn get(&self) -> Result<SyncMapItem, TwilioError> {
        let map_item = self.client.send_request::<SyncMapItem, ()>(
            Method::GET,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items/{}",
                self.service_sid, self.map_sid, self.key
            ),
            None,
        );

        map_item
    }

    /// [Update a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#update-a-mapitem-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and and the Map provided to the `Map` argument
    /// and updates the targeted key
    pub fn update(&self, params: UpdateParams) -> Result<SyncMapItem, TwilioError> {
        let map_item = self.client.send_request::<SyncMapItem, UpdateParams>(
            Method::POST,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items/{}",
                self.service_sid, self.map_sid, self.key
            ),
            Some(&params),
        );

        map_item
    }

    /// [Deletes a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#delete-a-mapitem-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and the Map provided to the `Map` argument
    /// and DELETES the targeted key.
    pub fn delete(&self) -> Result<(), TwilioError> {
        let service = self.client.send_request_and_ignore_response::<()>(
            Method::DELETE,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items/{}",
                self.service_sid, self.map_sid, self.key
            ),
            None,
        );

        service
    }
}
