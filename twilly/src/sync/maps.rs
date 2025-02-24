/*!

Contains Twilio Sync Map related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::mapitems::{MapItem, MapItems};

/// Represents a page of Sync Maps from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SyncMapPage {
    pub maps: Vec<SyncMap>,
    pub meta: PageMeta,
}

/// A Sync Map resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMap {
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

/// Resources _linked_ to a Sync Map
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub items: String,
    pub permissions: String,
}

/// Parameters for creating a Sync Map
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    pub unique_name: Option<String>,
    pub ttl: Option<bool>,
}

/// Parameters for updating a Sync Map
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct UpdateParams {
    pub ttl: Option<bool>,
}

pub struct Maps<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
}

impl<'a, 'b> Maps<'a, 'b> {
    /// [Creates a Sync Map resource](https://www.twilio.com/docs/sync/api/map-resource#create-a-syncmap-resource)
    ///
    /// Creates a Sync Map resource with the provided parameters.
    pub async fn create(&self, params: CreateParams) -> Result<SyncMap, TwilioError> {
        self.client
            .send_request::<SyncMap, CreateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps",
                    &self.service_sid
                ),
                Some(&params),
                None,
            )
            .await
    }

    /// [Lists Sync Maps](https://www.twilio.com/docs/sync/api/map-resource#read-multiple-syncmap-resources)
    ///
    /// Lists Sync Maps existing on the Twilio account.
    ///
    /// Maps will be _eagerly_ paged until all retrieved.
    pub async fn list(&self) -> Result<Vec<SyncMap>, TwilioError> {
        let mut maps_page = self
            .client
            .send_request::<SyncMapPage, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps?PageSize=20",
                    self.service_sid
                ),
                None,
                None,
            )
            .await?;

        let mut results: Vec<SyncMap> = maps_page.maps;

        while (maps_page.meta.next_page_url).is_some() {
            maps_page = self
                .client
                .send_request::<SyncMapPage, ()>(
                    Method::GET,
                    &maps_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut maps_page.maps);
        }

        Ok(results)
    }
}

pub struct Map<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    /// SID of the Sync Map. Can also be it's unique name.
    pub sid: &'b str,
}

impl<'a, 'b> Map<'a, 'b> {
    /// [Gets a Sync Map](https://www.twilio.com/docs/sync/api/map-resource#fetch-a-syncmap-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and fetches the Map
    /// provided to the `map()` argument.
    pub async fn get(&self) -> Result<SyncMap, TwilioError> {
        self.client
            .send_request::<SyncMap, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }

    /// [Update a Sync Map](https://www.twilio.com/docs/sync/api/map-resource#update-a-syncmap-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument  and updates the Map
    /// provided to the `map()` argument.
    pub async fn update(&self, params: UpdateParams) -> Result<SyncMap, TwilioError> {
        self.client
            .send_request::<SyncMap, UpdateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}",
                    self.service_sid, self.sid
                ),
                Some(&params),
                None,
            )
            .await
    }

    /// [Deletes a Sync Map](https://www.twilio.com/docs/sync/api/map-resource#delete-a-sync-map-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and deletes the Map
    /// provided to the `map()` argument.
    ///
    /// This will delete any Sync Map items underneath this map.
    pub async fn delete(&self) -> Result<(), TwilioError> {
        self.client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Maps/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }

    /// Functions relating to a known Sync Map Item.
    ///
    /// Takes in the key of the Sync Map Item to perform actions against.
    pub fn mapitem(&self, key: &'b str) -> MapItem {
        MapItem {
            client: self.client,
            service_sid: self.service_sid,
            map_sid: self.sid,
            key,
        }
    }

    /// General Sync Map Item functions.
    pub fn mapitems(&self) -> MapItems {
        MapItems {
            client: self.client,
            service_sid: self.service_sid,
            map_sid: self.sid,
        }
    }
}
