/*!

Contains Twilio Serverless Environment related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::{header::HeaderMap, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Represents a page of Serverless Environments from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct EnvironmentPage {
    environments: Vec<Environment>,
    meta: PageMeta,
}

/// A Serverless Environment resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub sid: String,
    pub account_sid: String,
    pub service_sid: String,
    pub build_sid: String,
    pub unique_name: String,
    /// URL-friendly name which forms part of the domain (unless production).
    pub domain_suffix: String,
    /// Domain for all functions & assets deployed in the Environment.
    pub domain_name: String,
    pub url: String,
    pub date_created: String,
    pub date_updated: String,
}

/// Resources _linked_ to a environment
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub variables: String,
    pub deployments: String,
    logs: String,
}

/// Parameters for creating an Environment
pub struct CreateParams<'a, T>
where
    T: ?Sized + Serialize,
{
    pub unique_name: Option<String>,
    pub data: &'a T,
    /// How long the Document should exist before deletion (in seconds).
    pub ttl: Option<u16>,
}

/// Parameters for creating a Sync Document with
/// data converted to a JSON string
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
struct CreateParamsWithJson {
    unique_name: Option<String>,
    data: String,
    /// How long the Document should exist before deletion (in seconds).
    ttl: Option<u16>,
}

/// Parameters for updating a Sync Document
pub struct UpdateParams<'a, T>
where
    T: ?Sized + Serialize,
{
    pub if_match: Option<String>,
    /// Any value that can be represented as JSON
    pub data: &'a T,
    /// How long the Document should exist before deletion (in seconds).
    pub ttl: Option<u16>,
}

/// Parameters for creating a Sync Document with
/// data converted to a JSON string
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
struct UpdateParamsWithJson {
    #[serde(rename(serialize = "If-Match"))]
    if_match: Option<String>,
    /// Any value that can be represented as JSON
    data: String,
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
    pub async fn create<T>(&self, params: CreateParams<'_, T>) -> Result<Environment, TwilioError>
    where
        T: ?Sized + Serialize,
    {
        let params = CreateParamsWithJson {
            unique_name: params.unique_name,
            data: serde_json::to_string(params.data)
                .expect("Unable to convert provided data value to a JSON string"),
            ttl: params.ttl,
        };

        self.client
            .send_request::<Environment, CreateParamsWithJson>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents",
                    self.service_sid
                ),
                Some(&params),
                None,
            )
            .await
    }

    /// [Lists Sync Documents](https://www.twilio.com/docs/sync/api/document-resource#read-multiple-document-resources)
    ///
    /// Lists Sync Documents in the Sync Service provided to the `service()`.
    ///
    /// Documents will be _eagerly_ paged until all retrieved.
    pub async fn list(&self) -> Result<Vec<Environment>, TwilioError> {
        let mut documents_page = self
            .client
            .send_request::<EnvironmentPage, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents?PageSize=50",
                    self.service_sid
                ),
                None,
                None,
            )
            .await?;

        let mut results: Vec<Environment> = documents_page.documents;

        while (documents_page.meta.next_page_url).is_some() {
            documents_page = self
                .client
                .send_request::<EnvironmentPage, ()>(
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
    pub async fn get(&self) -> Result<Environment, TwilioError> {
        self.client
            .send_request::<Environment, ()>(
                Method::GET,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }

    /// [Update a Sync Document](https://www.twilio.com/docs/sync/api/document-resource#update-a-document-resource)
    ///
    /// Targets the Sync Service provided to the `service()` argument and updates the Document
    /// provided to the `document()` argument.
    pub async fn update<T>(&self, params: UpdateParams<'_, T>) -> Result<Environment, TwilioError>
    where
        T: ?Sized + Serialize,
    {
        // Create a new struct with the provided data parameter converted to a
        // JSON string as required by Twilio.
        let params = UpdateParamsWithJson {
            if_match: params.if_match,
            data: serde_json::to_string(params.data)
                .expect("Unable to convert provided data value to a JSON string"),
            ttl: params.ttl,
        };

        let mut headers = HeaderMap::new();

        if let Some(if_match) = params.if_match.clone() {
            headers.append("If-Match", if_match.parse().unwrap());
        }

        self.client
            .send_request::<Environment, UpdateParamsWithJson>(
                Method::POST,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents/{}",
                    self.service_sid, self.sid
                ),
                Some(&params),
                Some(headers),
            )
            .await
    }

    /// [Deletes a Sync Service](https://www.twilio.com/docs/sync/api/service#delete-a-service-resourcee)
    ///
    /// Targets the Sync Service provided to the `service()` argument and deletes the Document
    /// provided to the `document()` argument.
    pub async fn delete(&self) -> Result<(), TwilioError> {
        self.client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!(
                    "https://sync.twilio.com/v1/Services/{}/Documents/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }
}
