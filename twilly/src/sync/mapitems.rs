/*!

Contains Twilio Sync Map Item related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::{header::HeaderMap, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Represents a page of Sync Map Items from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct MapItemPage {
    pub items: Vec<SyncMapItem>,
    pub meta: PageMeta,
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

/// Parameters for creating a Sync Map Item. Data must be a value
/// capable to converting to JSON in which all keys must be
/// strings.
pub struct CreateParams<'a, T>
where
    T: ?Sized + Serialize,
{
    pub key: String,
    /// Any value that can be represented as JSON
    pub data: &'a T,
    /// How long the Map Item should exist before deletion (in seconds).
    pub ttl: Option<u16>,
    /// How long the *parent* Map resource should exist before deletion (in seconds).
    pub collection_ttl: Option<u16>,
}

/// Parameters for creating a Sync Map Item with
/// data converted to a JSON string
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParamsWithJson {
    pub key: String,
    /// JSON string of data
    pub data: String,
    /// How long the Map Item should exist before deletion (in seconds).
    pub ttl: Option<u16>,
    /// How long the *parent* Map resource should exist before deletion (in seconds).
    pub collection_ttl: Option<u16>,
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

pub struct MapItems<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub map_sid: &'b str,
}

impl<'a, 'b> MapItems<'a, 'b> {
    /// [Creates a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#create-a-mapitem-resource)
    ///
    /// Creates a Sync Map Item with the provided parameters.
    pub async fn create<T>(&self, params: CreateParams<'_, T>) -> Result<SyncMapItem, TwilioError>
    where
        T: ?Sized + Serialize,
    {
        // Create a new struct with the provided data parameter converted to a
        // JSON string as required by Twilio.
        let params = CreateParamsWithJson {
            key: params.key,
            data: serde_json::to_string(params.data)
                .expect("Unable to convert provided data value to a JSON string"),
            ttl: params.ttl,
            collection_ttl: params.collection_ttl,
        };

        let map_item = self
            .client
            .send_request::<SyncMapItem, CreateParamsWithJson>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items",
                    self.service_sid, self.map_sid
                ),
                Some(&params),
                None,
            )
            .await;

        map_item
    }

    /// [Lists Sync Map Items](https://www.twilio.com/docs/sync/api/map-item-resource#read-all-mapitem-resources)
    ///
    /// List Sync Map Items In the targeted Service and Map.
    ///
    /// Targets the Sync Service provided to the `service()` argument, the Map provided to the `map()`
    /// argument and lists all Map items.
    ///
    /// Map items will be _eagerly_ paged until all retrieved.
    pub async fn list(&self, params: ListParams) -> Result<Vec<SyncMapItem>, TwilioError> {
        let mut map_items_page = self
            .client
            .send_request::<MapItemPage, ListParams>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items?PageSize=50",
                    self.service_sid, self.map_sid
                ),
                Some(&params),
                None,
            )
            .await?;

        let mut results: Vec<SyncMapItem> = map_items_page.items;

        while (map_items_page.meta.next_page_url).is_some() {
            map_items_page = self
                .client
                .send_request::<MapItemPage, ListParams>(
                    Method::GET,
                    &map_items_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

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
    /// Targets the Sync Service provided to the `service()` argument, the Map provided to the `map()`
    /// argument and fetches the item with the key provided to `mapitem()`.
    pub async fn get(&self) -> Result<SyncMapItem, TwilioError> {
        let map_item = self
            .client
            .send_request::<SyncMapItem, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items/{}",
                    self.service_sid, self.map_sid, self.key
                ),
                None,
                None,
            )
            .await;

        map_item
    }

    /// [Update a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#update-a-mapitem-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument, the Map provided to the `map()`
    /// argument and updates the item with the key provided to `mapitem()` with the parameters.
    pub async fn update(&self, params: UpdateParams) -> Result<SyncMapItem, TwilioError> {
        let mut headers = HeaderMap::new();

        if let Some(if_match) = params.if_match.clone() {
            headers.append("If-Match", if_match.parse().unwrap());
        }

        let map_item = self
            .client
            .send_request::<SyncMapItem, UpdateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items/{}",
                    self.service_sid, self.map_sid, self.key
                ),
                Some(&params),
                Some(headers),
            )
            .await;

        map_item
    }

    /// [Deletes a Sync Map Item](https://www.twilio.com/docs/sync/api/map-item-resource#delete-a-mapitem-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument, the Map provided to the `map()`
    /// argument and deletes the item with the key provided to `mapitem()`.
    pub async fn delete(&self) -> Result<(), TwilioError> {
        let map_item = self
            .client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}/Items/{}",
                    self.service_sid, self.map_sid, self.key
                ),
                None,
                None,
            )
            .await;

        map_item
    }
}
