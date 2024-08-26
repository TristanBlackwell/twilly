/*!

Contains Twilio Serverless Environment Logs related functionality.

*/

use crate::{Client, PageMeta, TwilioError};
use chrono::Utc;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

/// Represents a page of Serverless Environments from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct LogsPage {
    logs: Vec<ServerlessLog>,
    meta: PageMeta,
}

/// A Serverless Environment resource.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ServerlessLog {
    pub sid: String,
    pub account_sid: String,
    pub service_sid: String,
    pub environment_sid: String,
    pub build_sid: String,
    pub deployment_sid: String,
    pub function_sid: String,
    pub request_sid: String,
    pub level: Level,
    pub message: String,
    pub date_created: String,
    pub url: String,
}

#[derive(
    AsRefStr,
    Clone,
    Display,
    Default,
    Debug,
    EnumIter,
    EnumString,
    Serialize,
    Deserialize,
    PartialEq,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    #[default]
    #[strum(to_string = "Info")]
    Info,
    #[strum(to_string = "Warn")]
    Warn,
    #[strum(to_string = "Error")]
    Error,
}

/// Arguments for listing Serverless Logs
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct ListParams {
    // The SID of the specific function producing logs.
    pub function_sid: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub struct Logs<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub environment_sid: &'b str,
}

impl<'a, 'b> Logs<'a, 'b> {
    /// [Lists Logs of an Environment](https://www.twilio.com/docs/serverless/api/resource/logs#read-multiple-log-resources)
    ///
    /// Lists Logs of the Environment provided to `environment()` under the Serverless Service
    /// provided to the `service()`.
    ///
    /// Logs will be _eagerly_ paged until all retrieved. If `start_date` is None, this defaults to 1 day in the
    /// past. If `end_date` is None, this defaults to the current datetime.
    pub async fn list(
        &self,
        function_sid: Option<String>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<ServerlessLog>, TwilioError> {
        let params = ListParams {
            function_sid,
            start_date: start_date.map(|sd| sd.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            end_date: end_date.map(|ed| ed.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        };
        dbg!(&params.start_date);
        dbg!(&params.end_date);

        let mut logs_page = self
            .client
            .send_request::<LogsPage, ListParams>(
                Method::GET,
                &format!(
                    "https://serverless.twilio.com/v1/Services/{}/Environments/{}/Logs?PageSize=500",
                    self.service_sid, self.environment_sid
                ),
                Some(&params),
                None,
            )
            .await?;

        let mut results: Vec<ServerlessLog> = logs_page.logs;

        while (logs_page.meta.next_page_url).is_some() {
            logs_page = self
                .client
                .send_request::<LogsPage, ()>(
                    Method::GET,
                    &logs_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut logs_page.logs);
        }

        Ok(results)
    }
}

pub struct Log<'a, 'b> {
    pub client: &'a Client,
    pub service_sid: &'b str,
    pub environment_sid: &'b str,
    /// SID of the Log Resource.
    pub sid: &'b str,
}

impl<'a, 'b> Log<'a, 'b> {
    /// [Gets an Log](https://www.twilio.com/docs/serverless/api/resource/logs#fetch-a-log-resource)
    ///
    /// Targets the Serverless Service provided to the `service()` argument and the Environment provided to
    /// the `environment()` argument and fetches a Log provided to the `log()` argument.
    pub async fn get(&self) -> Result<ServerlessLog, TwilioError> {
        self.client
            .send_request::<ServerlessLog, ()>(
                Method::GET,
                &format!(
                    "https://serverless.twilio.com/v1/Services/{}/Environments/{}/Logs/{}",
                    self.service_sid, self.environment_sid, self.sid
                ),
                None,
                None,
            )
            .await
    }
}
