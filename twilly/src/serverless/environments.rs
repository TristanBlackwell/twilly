/*!

Contains Twilio Serverless Environment related functionality.

*/

pub mod logs;

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Represents a page of Serverless Environments from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct EnvironmentPage {
    environments: Vec<ServerlessEnvironment>,
    meta: PageMeta,
}

/// A Serverless Environment resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerlessEnvironment {
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

/// Resources _linked_ to a environment.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub variables: String,
    pub deployments: String,
    logs: String,
}

/// Parameters for creating an Environment.
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateParams {
    pub unique_name: String,
    /// URL-friendly name that forms part of the domain name.
    pub domain_suffix: Option<String>,
}

pub struct Environments<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
}

impl<'a, 'b> Environments<'a, 'b> {
    /// [Creates an Environment](https://www.twilio.com/docs/serverless/api/resource/environment#create-an-environment-resource)
    ///
    /// Creates an Environment with the provided parameters.
    pub async fn create<T>(
        &self,
        params: CreateParams,
    ) -> Result<ServerlessEnvironment, TwilioError>
    where
        T: ?Sized + Serialize,
    {
        self.client
            .send_request::<ServerlessEnvironment, CreateParams>(
                Method::POST,
                &format!(
                    "https://serverless.twilio.com/v1/Services/{}/Environments",
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
    pub async fn list(&self) -> Result<Vec<ServerlessEnvironment>, TwilioError> {
        let mut environments_page = self
            .client
            .send_request::<EnvironmentPage, ()>(
                Method::GET,
                &format!(
                    "https://serverless.twilio.com/v1/Services/{}/Environments?PageSize=50",
                    self.service_sid
                ),
                None,
                None,
            )
            .await?;

        let mut results: Vec<ServerlessEnvironment> = environments_page.environments;

        while (environments_page.meta.next_page_url).is_some() {
            environments_page = self
                .client
                .send_request::<EnvironmentPage, ()>(
                    Method::GET,
                    &environments_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut environments_page.environments);
        }

        Ok(results)
    }
}

pub struct Environment<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    /// SID of the Environment.
    pub sid: &'b str,
}

impl<'a, 'b> Environment<'a, 'b> {
    /// [Gets an Environment](https://www.twilio.com/docs/serverless/api/resource/environment#fetch-an-environment-resource)
    ///
    /// Targets the Serverless Service provided to the `service()` argument and fetches the Environment
    /// provided to the `environment()` argument.
    pub async fn get(&self) -> Result<ServerlessEnvironment, TwilioError> {
        self.client
            .send_request::<ServerlessEnvironment, ()>(
                Method::GET,
                &format!(
                    "https://serverless.twilio.com/v1/Services/{}/Environments/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }

    /// [Deletes an Environment](https://www.twilio.com/docs/serverless/api/resource/environment#delete-an-environment-resource)
    ///
    /// Targets the Serverless Service provided to the `service()` argument and deletes the Environment
    /// provided to the `environment()` argument.
    pub async fn delete(&self) -> Result<(), TwilioError> {
        self.client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!(
                    "https://serverless.twilio.com/v1/Services/{}/Environments/{}",
                    self.service_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }
}
