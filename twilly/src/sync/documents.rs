/*!

Contains Twilio Sync Document related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::{header::HeaderMap, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Represents a page of Sync Documents from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct DocumentPage {
    documents: Vec<SyncDocument>,
    meta: PageMeta,
}

/// A Sync Document resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncDocument {
    pub sid: String,
    pub unique_name: String,
    pub account_sid: String,
    pub service_sid: String,
    pub url: String,
    pub data: Value,
    pub date_created: String,
    pub date_updated: String,
    pub date_expires: Option<String>,
    /// Identity of the creator. Uses the identity of the
    /// respective client or defaults to `system` if created via REST.
    pub created_by: String,
    pub links: Links,
    pub revision: String,
}

/// Resources _linked_ to a document
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub permissions: String,
}

impl Default for Links {
    fn default() -> Self {
        Links {
            permissions: String::from(""),
        }
    }
}

/// Parameters for creating a Sync Document
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    unique_name: Option<String>,
    data: Value,
    /// How long the Document should exist before deletion (in seconds).
    ttl: Option<u16>,
}

/// Parameters for updating a Sync Service
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct UpdateParams {
    #[serde(rename(serialize = "If-Match"))]
    if_match: Option<String>,
    data: Value,
    /// How long the Document should exist before deletion (in seconds).
    ttl: Option<u16>,
}

pub struct Documents<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
}

impl<'a, 'b> Documents<'a, 'b> {
    /// [Creates a Sync Document](https://www.twilio.com/docs/sync/api/document-resource)
    ///
    /// Creates a Sync Document with the provided parameters.
    pub async fn create(&self, params: CreateParams) -> Result<SyncDocument, TwilioError> {
        let document = self
            .client
            .send_request::<SyncDocument, CreateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents",
                    self.service_sid
                ),
                Some(&params),
                None,
            )
            .await;

        document
    }

    /// [Lists Sync Documents](https://www.twilio.com/docs/sync/api/document-resource#read-multiple-document-resources)
    ///
    /// Lists Sync Documents in the Sync Service provided to the `service()`.
    ///
    /// Documents will be _eagerly_ paged until all retrieved.
    pub async fn list(&self) -> Result<Vec<SyncDocument>, TwilioError> {
        let mut documents_page = self
            .client
            .send_request::<DocumentPage, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents?PageSize=50",
                    self.service_sid
                ),
                None,
                None,
            )
            .await?;

        let mut results: Vec<SyncDocument> = documents_page.documents;

        while (documents_page.meta.next_page_url).is_some() {
            documents_page = self
                .client
                .send_request::<DocumentPage, ()>(
                    Method::GET,
                    &documents_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut documents_page.documents);
        }

        Ok(results)
    }
}

pub struct Document<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    /// SID of the Sync Document. Can also be the friendly name.
    pub sid: &'b str,
}

impl<'a, 'b> Document<'a, 'b> {
    /// [Gets a Sync Document](https://www.twilio.com/docs/sync/api/document-resource#fetch-a-document-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and fetches the Document
    /// provided to the `document()` argument.
    pub async fn get(&self) -> Result<SyncDocument, TwilioError> {
        let document = self
            .client
            .send_request::<SyncDocument, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await;

        document
    }

    /// [Update a Sync Document](https://www.twilio.com/docs/sync/api/document-resource#update-a-document-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and updates the Document
    /// provided to the `document()` argument.
    pub async fn update(&self, params: UpdateParams) -> Result<SyncDocument, TwilioError> {
        let mut headers = HeaderMap::new();

        if let Some(if_match) = params.if_match.clone() {
            headers.append("If-Match", if_match.parse().unwrap());
        }

        let document = self
            .client
            .send_request::<SyncDocument, UpdateParams>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents/{}",
                    self.service_sid, self.sid
                ),
                Some(&params),
                Some(headers),
            )
            .await;

        document
    }

    /// [Deletes a Sync Service](https://www.twilio.com/docs/sync/api/service#delete-a-service-resourcee)
    ///
    /// Targets the Sync Service provided to the `service()` argument and deletes the Document
    /// provided to the `document()` argument.
    pub async fn delete(&self) -> Result<(), TwilioError> {
        let service = self
            .client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await;

        service
    }
}
