/*!

Contains Twilio conversations related functionality.

*/
use std::fmt;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use crate::{Client, PageMeta, TwilioError};

/// Holds conversation related functions accessible
/// on the client.
pub struct Conversations<'a> {
    pub client: &'a Client,
}

/// Represents a page of conversations from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ConversationPage {
    conversations: Vec<Conversation>,
    meta: PageMeta,
}

/// Details related to a specific conversation.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversation {
    pub sid: String,
    pub account_sid: String,
    pub chat_service_sid: String,
    pub messaging_service_sid: String,
    pub unique_name: Option<String>,
    pub friendly_name: Option<String>,
    pub date_created: String,
    pub date_updated: String,
    pub state: State,
    pub url: String,
    pub attributes: String,
    pub timers: Timers,
    pub links: Links,
}

impl fmt::Display for Conversation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.sid, self.state)
    }
}

/// Details related to a specific conversation.
#[derive(Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct UpdateConversation {
    pub unique_name: Option<String>,
    pub friendly_name: Option<String>,
    pub state: Option<State>,
    pub attributes: Option<String>,
    pub timers: Option<Timers>,
}

/// The possible states of a conversation.
#[derive(
    AsRefStr, Clone, Display, Debug, EnumIter, EnumString, Serialize, Deserialize, PartialEq,
)]
#[serde(rename_all = "lowercase")]
pub enum State {
    #[strum(to_string = "Active")]
    Active,
    #[strum(to_string = "Inactive")]
    Inactive,
    #[strum(to_string = "Closed")]
    Closed,
}

impl Default for State {
    fn default() -> Self {
        State::Active
    }
}

impl State {
    pub fn as_str(&self) -> &'static str {
        match self {
            &State::Active => "active",
            &State::Inactive => "inactive",
            &State::Closed => "closed",
        }
    }
}

/// The timers configured for a conversation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Timers {
    #[serde(rename(serialize = "Timers.Inactive"))]
    pub date_inactive: Option<String>,
    #[serde(rename(serialize = "Timers.Closed"))]
    pub date_closed: Option<String>,
}

impl Default for Timers {
    fn default() -> Self {
        Timers {
            date_inactive: None,
            date_closed: None,
        }
    }
}

/// Links to resources _linked_ to a conversation
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub participants: String,
    pub messages: String,
    pub webhooks: String,
}

impl Default for Links {
    fn default() -> Self {
        Links {
            participants: String::from(""),
            messages: String::from(""),
            webhooks: String::from(""),
        }
    }
}

/// Possible filters when listing Conversations via the Twilio API
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
struct ListParams {
    start_date: Option<String>,
    end_date: Option<String>,
    state: Option<State>,
}

impl<'a> Conversations<'a> {
    /// [Gets a Conversation](https://www.twilio.com/docs/conversations/api/conversation-resource#fetch-a-conversation-resource)
    ///
    /// Takes in a `sid` argument which can also be the conversations `uniqueName`.
    pub fn get(&self, sid: &str) -> Result<Conversation, TwilioError> {
        let conversation = self.client.send_request::<Conversation, ()>(
            Method::GET,
            &format!("https://conversations.twilio.com/v1/Conversations/{}", sid),
            None,
        );

        conversation
    }

    /// [Lists Conversations](https://www.twilio.com/docs/conversations/api/conversation-resource#read-multiple-conversation-resources)
    ///
    /// This will eagerly fetch *all* conversations on the Twilio account and sort by recent message activity.
    /// Takes in `start_date` and `end_date` options to filter results. This should be ISO8601 format e.g. `YYYY-MM-DDT00:00:00Z`.
    ///
    /// Also accepts a `state` option to filter by Conversation state such as closed Conversations.
    pub fn list(
        &self,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        state: Option<State>,
    ) -> Result<Vec<Conversation>, TwilioError> {
        let params = ListParams {
            start_date: if let Some(start_date) = start_date {
                Some(start_date.to_string())
            } else {
                None
            },
            end_date: if let Some(end_date) = end_date {
                Some(end_date.to_string())
            } else {
                None
            },
            state,
        };

        let mut conversations_page = self.client.send_request::<ConversationPage, ListParams>(
            Method::GET,
            "https://conversations.twilio.com/v1/Conversations",
            Some(&params),
        )?;

        let mut results: Vec<Conversation> = conversations_page.conversations;

        while (conversations_page.meta.next_page_url).is_some() {
            conversations_page = self.client.send_request::<ConversationPage, ()>(
                Method::GET,
                &conversations_page.meta.next_page_url.unwrap(),
                None,
            )?;

            results.append(&mut conversations_page.conversations);
        }

        Ok(results)
    }

    /// [Update a Conversation](https://www.twilio.com/docs/conversations/api/conversation-resource#update-conversation)
    ///
    /// Takes in a `sid` argument which can also be the conversations `uniqueName` and updates the resource with the
    /// provided properties.
    pub fn update(
        &self,
        sid: &str,
        updates: UpdateConversation,
    ) -> Result<Conversation, TwilioError> {
        let conversation = self
            .client
            .send_request::<Conversation, UpdateConversation>(
                Method::POST,
                &format!("https://conversations.twilio.com/v1/Conversations/{}", sid),
                Some(&updates),
            );

        conversation
    }

    /// [Deletes a Conversation](https://www.twilio.com/docs/conversations/api/conversation-resource#delete-a-conversation-resource)
    ///
    /// Takes in a `sid` argument which can also be the conversations `uniqueName` and **deletes** the resource.
    pub fn delete(&self, sid: &str) -> Result<(), TwilioError> {
        let conversation = self.client.send_request_and_ignore_response::<()>(
            Method::DELETE,
            &format!("https://conversations.twilio.com/v1/Conversations/{}", sid),
            None,
        );

        conversation
    }
}
