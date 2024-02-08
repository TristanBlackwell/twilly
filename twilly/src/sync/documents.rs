/*!

Contains Twilio Sync related functionality.

*/

use crate::{Client, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Holds Sync related functions accessible
/// on the client.
pub struct Documents<'a> {
    pub client: &'a Client,
}

/// Represents a page of Sync Services from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct DocumentPage {
    documents: Vec<Document>,
    meta: DocumentPageMeta,
}

/// Holds the actual page information from the API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct DocumentPageMeta {
    page: u16,
    page_size: u16,
    first_page_url: String,
    previous_page_url: Option<String>,
    next_page_url: Option<String>,
    key: String,
}

/// A Sync Service resource.
#[derive(Serialize, Deserialize)]
pub struct Document {
    pub sid: String,
    pub unique_name: String,
    pub account_sid: String,
    pub service_sid: String,
    pub url: String,
    pub data: String,
    pub date_created: String,
    pub date_updated: String,
    pub date_expires: String,
    /// Identity of the Document creator. Uses the identity of the
    /// respective client or defaults to `system` if created via REST.
    pub created_by: String,
    pub links: Links,
    pub revision: String,
}

/// Links to resources _linked_ to a conversation
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

/// Arguments for creating or updating a Sync Service
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateOrUpdateParams {
    friendly_name: Option<String>,
    webhook_url: Option<String>,
    reachability_webhooks_enabled: Option<bool>,
    acl_enabled: Option<bool>,
    reachability_debouncing_enabled: Option<bool>,
    reachability_debouncing_window: Option<u16>,
    webhooks_from_rest_enabled: Option<bool>,
}

pub struct Services<'a> {
    pub client: &'a Client,
}

impl<'a> Services<'a> {
    /// [Creates a Sync Service](https://www.twilio.com/docs/sync/api/service#create-a-service-resource)
    ///
    /// Takes in an `sid` argument of the Sync Service to fetch. Can also be the unique name
    pub fn create(&self, params: CreateOrUpdateParams) -> Result<Document, TwilioError> {
        if let Some(reachability_debouncing_window) = params.reachability_debouncing_window {
            let validation =
                validate_reachability_debouncing_window(reachability_debouncing_window);

            if validation.is_err() {
                return Err(validation.unwrap_err());
            }
        }

        let service = self.client.send_request::<Document, CreateOrUpdateParams>(
            Method::GET,
            "https://sync.twilio.com/v1/Services",
            Some(&params),
        );

        service
    }

    /// [Lists Sync Services](https://www.twilio.com/docs/sync/api/service#read-multiple-service-resources)
    ///
    /// This will list Sync Services existing on the Twilio account.
    ///
    /// Services will be _eagerly_ paged until all retrieved.
    ///
    /// Takes optional parameters:
    /// - `friendly_name` - Return only accounts matching this friendly name
    /// - `status` - Return only accounts that match this status
    pub fn list(&self) -> Result<Vec<Document>, TwilioError> {
        let mut services_page = self.client.send_request::<DocumentPage, ()>(
            Method::GET,
            "https://sync.twilio.com/v1/Services?PageSize=20",
            None,
        )?;

        let mut results: Vec<Document> = services_page.services;

        while (services_page.meta.next_page_url).is_some() {
            services_page = self.client.send_request::<DocumentPage, ()>(
                Method::GET,
                &services_page.meta.next_page_url.unwrap(),
                None,
            )?;

            results.append(&mut services_page.services);
        }

        Ok(results)
    }
}

pub struct Service<'a> {
    pub client: &'a Client,
    pub sid: String,
}

impl<'a> Service<'a> {
    /// [Gets a Sync Service](https://www.twilio.com/docs/sync/api/service#fetch-a-service-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and fetches the resource
    pub fn get(&self) -> Result<Document, TwilioError> {
        let service = self.client.send_request::<Document, ()>(
            Method::GET,
            &format!("https://sync.twilio.com/v1/Services/{}", self.sid),
            None,
        );

        service
    }

    /// [Update a Sync Service](https://www.twilio.com/docs/sync/api/service#update-a-service-resource)
    ///
    /// Targets the Sync Service provided to the `Service` argument and updates the resource with
    /// the provided properties
    pub fn update(&self, params: CreateOrUpdateParams) -> Result<Document, TwilioError> {
        if let Some(reachability_debouncing_window) = params.reachability_debouncing_window {
            let validation =
                validate_reachability_debouncing_window(reachability_debouncing_window);

            if validation.is_err() {
                return Err(validation.unwrap_err());
            }
        }

        let service = self.client.send_request::<Document, CreateOrUpdateParams>(
            Method::POST,
            &format!(
                "https://conversations.twilio.com/v1/Conversations/{}",
                self.sid
            ),
            Some(&params),
        );

        service
    }

    /// [Deletes a Sync Service](https://www.twilio.com/docs/sync/api/service#delete-a-service-resourcee)
    ///
    /// Targets the Sync Service provided to the `Service` argument and deletes the resource.
    pub fn delete(&self) -> Result<(), TwilioError> {
        let service = self.client.send_request_and_ignore_response::<()>(
            Method::DELETE,
            &format!("https://sync.twilio.com/v1/Services/{}", self.sid),
            None,
        );

        service
    }
}

/// Validates that the provided `reachability_debouncing_window` is between it's
/// expected millisecond values.
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
