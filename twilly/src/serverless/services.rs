/*!

Contains Twilio Serverless related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::environments::{Environment, Environments};

/// Represents a page of Services from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ServerlessServicePage {
    services: Vec<ServerlessService>,
    meta: PageMeta,
}

/// A Serverless Service resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerlessService {
    pub sid: String,
    pub account_sid: String,
    pub unique_name: String,
    pub friendly_name: String,
    /// Whether account credentials are accessible in function executions.
    pub include_credentials: bool,
    /// Whether code and configuration can be controlled from the Twilio Console.
    pub ui_editable: bool,
    /// The base domain name of the service (combination of `unique_name` and random numbers)
    pub domain_base: String,
    pub date_created: String,
    pub date_updated: String,
    pub url: String,
    pub links: Links,
}

/// Resources _linked_ to a Service
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub environments: String,
    pub functions: String,
    pub assets: String,
    pub builds: String,
}

/// Parameters for creating or updating a Serverlesss Service. See `ServerlessService` for
/// details on individual parameters.
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct CreateOrUpdateParams {
    pub unique_name: String,
    pub friendly_name: String,
    pub include_credentials: Option<bool>,
    pub ui_editable: Option<bool>,
}

pub struct Services<'a> {
    pub client: &'a Client,
}

impl<'a> Services<'a> {
    /// [Creates a Serverless Service](https://www.twilio.com/docs/serverless/api/resource/service#create-a-service-resource)
    ///
    /// Creates a Serverless Service resource with the provided parameters.
    pub async fn create(
        &self,
        params: CreateOrUpdateParams,
    ) -> Result<ServerlessService, TwilioError> {
        self.client
            .send_request::<ServerlessService, CreateOrUpdateParams>(
                Method::POST,
                "https://serverless.twilio.com/v1/Services",
                Some(&params),
                None,
            )
            .await
    }

    //// [Lists Serverless Services](https://www.twilio.com/docs/serverless/api/resource/service#read-multiple-service-resources)
    ///
    /// List Serverless Services existing on the Twilio account.
    ///
    /// Services will be _eagerly_ paged until all retrieved.
    pub async fn list(&self) -> Result<Vec<ServerlessService>, TwilioError> {
        let mut services_page = self
            .client
            .send_request::<ServerlessServicePage, ()>(
                Method::GET,
                "https://serverless.twilio.com/v1/Services?PageSize=20",
                None,
                None,
            )
            .await?;

        let mut results: Vec<ServerlessService> = services_page.services;

        while (services_page.meta.next_page_url).is_some() {
            services_page = self
                .client
                .send_request::<ServerlessServicePage, ()>(
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
    /// [Gets a Serverless Service](https://www.twilio.com/docs/serverless/api/resource/service#fetch-a-service-resource)
    ///
    /// Fetches the Serverless Service provided to the `Service()`.
    pub async fn get(&self) -> Result<ServerlessService, TwilioError> {
        self.client
            .send_request::<ServerlessService, ()>(
                Method::GET,
                &format!("https://serverless.twilio.com/v1/Services/{}", self.sid),
                None,
                None,
            )
            .await
    }

    /// [Update a Serverless Service](https://www.twilio.com/docs/serverless/api/resource/service#update-a-service-resource)
    ///
    /// Targets the Serverless Service provided to the `Service()` argument and updates the resource with
    /// the provided properties
    pub async fn update(
        &self,
        params: CreateOrUpdateParams,
    ) -> Result<ServerlessService, TwilioError> {
        self.client
            .send_request::<ServerlessService, CreateOrUpdateParams>(
                Method::POST,
                &format!("https://serverless.twilio.com/v1/Services/{}", self.sid),
                Some(&params),
                None,
            )
            .await
    }

    /// [Deletes a Serverless Service](https://www.twilio.com/docs/serverless/api/resource/service#delete-a-service-resource)
    ///
    /// Targets the Serverless Service provided to the `Service()` argument and deletes the resource.
    /// **Use with caution.**
    pub async fn delete(&self) -> Result<(), TwilioError> {
        self.client
            .send_request_and_ignore_response::<()>(
                Method::DELETE,
                &format!("https://serverless.twilio.com/v1/Services/{}", self.sid),
                None,
                None,
            )
            .await
    }

    /// Actions relating to a known Service Environment.
    ///
    /// Takes in the SID of the Environment to perform actions against.
    pub fn environment(&'a self, sid: &'b str) -> Environment {
        Environment {
            client: self.client,
            service_sid: self.sid,
            sid,
        }
    }

    /// General Service Environment actions.
    pub fn environments(&'a self) -> Environments {
        Environments {
            client: self.client,
            service_sid: self.sid,
        }
    }
}
