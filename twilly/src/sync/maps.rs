/*!

Contains Twilio Sync Map related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Represents a page of Sync Maps from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SyncMapPage {
    maps: Vec<SyncMap>,
    meta: PageMeta,
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
    /// Identity of the Document creator. Uses the identity of the
    /// respective client or defaults to `system` if created via REST.
    pub created_by: String,
    pub links: Links,
    pub revision: String,
}

/// Links to resources _linked_ to a Sync Map
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

/// Arguments for creating a Sync Map
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    unique_name: Option<String>,
    ttl: Option<bool>,
}

/// Arguments for updating a Sync Map
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct UpdateParams {
    ttl: Option<bool>,
}

pub struct Maps<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
}

impl<'a, 'b> Maps<'a, 'b> {
    /// [Creates a Sync Map resource](https://www.twilio.com/docs/sync/api/map-resource#create-a-syncmap-resource)
    ///
    /// Creates a Sync Map resource with the provided parameters.
    pub fn create(&self, params: CreateParams) -> Result<SyncMap, TwilioError> {
        let service = self.client.send_request::<SyncMap, CreateParams>(
            Method::POST,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps",
                &self.service_sid
            ),
            Some(&params),
        );

        service
    }

    /// [Lists Sync Maps](https://www.twilio.com/docs/sync/api/map-resource#read-multiple-syncmap-resources)
    ///
    /// This will list Sync Maps existing on the Twilio account.
    ///
    /// Maps will be _eagerly_ paged until all retrieved.
    pub fn list(&self) -> Result<Vec<SyncMap>, TwilioError> {
        let mut services_page = self.client.send_request::<SyncMapPage, ()>(
            Method::GET,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps?PageSize=20",
                self.service_sid
            ),
            None,
        )?;

        let mut results: Vec<SyncMap> = services_page.maps;

        while (services_page.meta.next_page_url).is_some() {
            services_page = self.client.send_request::<SyncMapPage, ()>(
                Method::GET,
                &services_page.meta.next_page_url.unwrap(),
                None,
            )?;

            results.append(&mut services_page.maps);
        }

        Ok(results)
    }
}

pub struct Map<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    /// SID of the Sync Map
    pub sid: &'b str,
}

impl<'a, 'b> Map<'a, 'b> {
    /// [Gets a Sync Map](https://www.twilio.com/docs/sync/api/map-resource#fetch-a-syncmap-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and fetches the map resource described
    /// by `sid`. Can also be the unique name.
    pub fn get(&self) -> Result<SyncMap, TwilioError> {
        let service = self.client.send_request::<SyncMap, ()>(
            Method::GET,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}",
                self.service_sid, self.sid
            ),
            None,
        );

        service
    }

    /// [Update a Sync Map](https://www.twilio.com/docs/sync/api/map-resource#update-a-syncmap-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and updates the resource with
    /// the provided properties
    pub fn update(&self, params: UpdateParams) -> Result<SyncMap, TwilioError> {
        let service = self.client.send_request::<SyncMap, UpdateParams>(
            Method::POST,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}",
                self.service_sid, self.sid
            ),
            Some(&params),
        );

        service
    }

    /// [Deletes a Sync Map](https://www.twilio.com/docs/sync/api/map-resource#delete-a-sync-map-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and the map described
    /// by `sid` and deletes the resource.
    pub fn delete(&self) -> Result<(), TwilioError> {
        let service = self.client.send_request_and_ignore_response::<()>(
            Method::DELETE,
            &format!(
                "https://sync.twilio.com/v1/Services/{}/Maps/{}",
                self.service_sid, self.sid
            ),
            None,
        );

        service
    }
}