/*!

Contains Twilio Sync Service related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{
    documents::{Document, Documents},
    lists::{List, Lists},
    maps::{Map, Maps},
};

/// Represents a page of Sync Services from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SyncServicePage {
    services: Vec<SyncService>,
    meta: PageMeta,
}

/// A Sync Service resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncService {
    pub sid: String,
    pub unique_name: Option<String>,
    pub account_sid: String,
    pub friendly_name: Option<String>,
    pub date_created: String,
    pub date_updated: String,
    pub url: String,
    pub webhook_url: Option<String>,
    pub webhooks_from_rest_enabled: bool,
    /// Requires identities to be granted access to the Sync Service
    pub acl_enabled: bool,
    /// Whether the `endpoint_disconnected` webhook should occur after a
    /// specified delay period or immediately on disconnection. This gives clients
    /// the opportunity to re-connect without the event being fired. Defaults to `false`.
    pub reachability_debouncing_enabled: bool,
    /// The delay (in milliseconds) the Service will wait to send an `endpoint_disconnected`
    /// event when the last connected client disconnects. Defaults to `5000` but can range
    /// between `1000` and `30000`.
    pub reachability_debouncing_window: u16,
    pub links: Links,
}

/// Resources _linked_ to a Service
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub documents: String,
    pub lists: String,
    pub maps: String,
    pub streams: String,
}

impl Default for Links {
    fn default() -> Self {
        Links {
            documents: String::from(""),
            lists: String::from(""),
            maps: String::from(""),
            streams: String::from(""),
        }
    }
}

/// Parameters for creating or updating a Sync Service. See `SyncService` for
/// details on individual parameters.
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateOrUpdateParams {
    pub friendly_name: Option<String>,
    pub webhook_url: Option<String>,
    pub reachability_webhooks_enabled: Option<bool>,
    pub acl_enabled: Option<bool>,
    pub reachability_debouncing_enabled: Option<bool>,
    pub reachability_debouncing_window: Option<u16>,
    pub webhooks_from_rest_enabled: Option<bool>,
}

pub struct Services<'a> {
    pub client: &'a Client,
}

impl<'a> Services<'a> {
    /// [Creates a Sync Service](https://www.twilio.com/docs/sync/api/service#create-a-service-resource)
    ///
    /// Creates a Sync Service resource with the provided parameters.
    pub async fn create(&self, params: CreateOrUpdateParams) -> Result<SyncService, TwilioError> {
        if let Some(reachability_debouncing_window) = params.reachability_debouncing_window {
            let validation =
                validate_reachability_debouncing_window(reachability_debouncing_window);

            if validation.is_err() {
                return Err(validation.unwrap_err());
            }
        }

        let service = self
            .client
            .send_request::<SyncService, CreateOrUpdateParams>(
                Method::POST,
                "https://sync.twilio.com/v1/Services",
                Some(&params),
                None,
            )
            .await;

        service
    }

    /// [Lists Sync Services](https://www.twilio.com/docs/sync/api/service#read-multiple-service-resources)
    ///
    /// List Sync Services existing on the Twilio account.
    ///
    /// Services will be _eagerly_ paged until all retrieved.
    pub async fn list(&self) -> Result<Vec<SyncService>, TwilioError> {
        let mut services_page = self
            .client
            .send_request::<SyncServicePage, ()>(
                Method::GET,
                "https://sync.twilio.com/v1/Services?PageSize=20",
                None,
                None,
            )
            .await?;

        let mut results: Vec<SyncService> = services_page.services;

        while (services_page.meta.next_page_url).is_some() {
            services_page = self
                .client
                .send_request::<SyncServicePage, ()>(
                    Method::GET,
                    &services_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut services_page.services);
        }

        Ok(results)
    }
}

pub struct Service<'a, 'b> {
    pub client: &'a Client,
    pub sid: &'b str,
}

impl<'a, 'b> Service<'a, 'b> {
    /// [Gets a Sync Service](https://www.twilio.com/docs/sync/api/service#fetch-a-service-resource)
    ///
    /// Fetches the Sync Service provided to the `Service()`.
    pub async fn get(&self) -> Result<SyncService, TwilioError> {
        let service = self
            .client
            .send_request::<SyncService, ()>(
                Method::GET,
                &format!("https://sync.twilio.com/v1/Services/{}", self.sid),
                None,
                None,
            )
            .await;

        service
    }

    /// [Update a Sync Service](https://www.twilio.com/docs/sync/api/service#update-a-service-resource)
    ///
    /// Targets the Sync Service provided to the `Service()` argument and updates the resource with
    /// the provided properties
    pub async fn update(&self, params: CreateOrUpdateParams) -> Result<SyncService, TwilioError> {
        if let Some(reachability_debouncing_window) = params.reachability_debouncing_window {
            let validation =
                validate_reachability_debouncing_window(reachability_debouncing_window);

            if validation.is_err() {
                return Err(validation.unwrap_err());
            }
        }

        let service = self
            .client
            .send_request::<SyncService, CreateOrUpdateParams>(
                Method::POST,
                &format!("https://sync.twilio.com/v1/Services/{}", self.sid),
                Some(&params),
                None,
            )
            .await;

        service
    }

    /// [Deletes a Sync Service](https://www.twilio.com/docs/sync/api/service#delete-a-service-resource)
    ///
    /// Targets the Sync Service provided to the `Service()` argument and deletes the resource.
    /// **Use with caution. All sub resources (documents, maps, ...) will also be removed.**
    pub async fn delete(&self) -> Result<(), TwilioError> {
        let service = self
            .client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!("https://sync.twilio.com/v1/Services/{}", self.sid),
                None,
                None,
            )
            .await;

        service
    }

    /// Functions relating to a known Sync Document.
    ///
    /// Takes in the SID of the Sync Document to perform actions against.
    pub fn document(&'a self, sid: &'b str) -> Document {
        Document {
            client: self.client,
            service_sid: self.sid,
            sid,
        }
    }

    /// General Sync Document functions.
    pub fn documents(&'a self) -> Documents {
        Documents {
            client: self.client,
            service_sid: self.sid,
        }
    }

    /// Functions relating to a known Sync Map.
    ///
    /// Takes in the SID of the Sync Map to perform actions against.
    pub fn map(&'a self, sid: &'b str) -> Map {
        Map {
            client: self.client,
            service_sid: self.sid,
            sid,
        }
    }

    /// General Sync Map functions.
    pub fn maps(&'a self) -> Maps {
        Maps {
            client: self.client,
            service_sid: self.sid,
        }
    }

    /// General Sync List functions.
    pub fn lists(&'a self) -> Lists {
        Lists {
            client: self.client,
            service_sid: self.sid,
        }
    }

    /// Functions relating to a known Sync List.
    ///
    /// Takes in the SID of the Sync List to perform actions against.
    pub fn list(&'a self, sid: &'b str) -> List {
        List {
            client: self.client,
            service_sid: self.sid,
            sid,
        }
    }
}

// Validates that the provided `reachability_debouncing_window` is between it's
// expected millisecond values.
fn validate_reachability_debouncing_window(
    reachability_debouncing_window: u16,
) -> Result<(), TwilioError> {
    if reachability_debouncing_window < 1000 {
        return Err(TwilioError {
            kind: crate::ErrorKind::ValidationError(String::from(
                "Reachability debouncing window must be greater than 1000 milliseconds",
            )),
        });
    } else if reachability_debouncing_window > 30000 {
        return Err(TwilioError {
            kind: crate::ErrorKind::ValidationError(String::from(
                "Reachability debouncing window must be less than 30,000 milliseconds",
            )),
        });
    }

    Ok(())
}
