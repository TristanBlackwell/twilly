/*!

Contains Twilio participant conversation related functionality.

*/

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{
    conversation::{State, Timers},
    Client, PageMeta, TwilioError,
};

/// Holds participant conversation related functions accessible
/// on the client.
pub struct ParticipantConversations<'a> {
    pub client: &'a Client,
}

/// Represents a page of participant conversations from the Twilio API.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ParticipantConversationPage {
    conversations: Vec<ParticipantConversation>,
    meta: PageMeta,
}

/// Participant conversation details.
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct ParticipantConversation {
    pub account_sid: String,
    pub chat_service_sid: String,
    pub participant_sid: String,
    pub participant_user_sid: Option<String>,
    pub participant_identity: Option<String>,
    pub participant_messaging_binding: Option<ParticipantMessagingBinding>,
    pub conversation_sid: String,
    pub conversation_unique_name: Option<String>,
    pub conversation_friendly_name: Option<String>,
    pub conversation_attributes: String,
    pub conversation_date_created: String,
    pub conversation_date_updated: String,
    pub conversation_created_by: String,
    pub conversation_state: State,
    pub conversation_timers: Timers,
    pub links: Links,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct ParticipantMessagingBinding {
    pub address: String,
    pub proxy_address: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub level: Option<String>,
    pub name: Option<String>,
    pub projected_address: Option<String>,
}

/// Resources _linked_ to a participants conversation. These can be used to retrieve
/// sub resources directly.
#[derive(Clone, Default, Debug, Deserialize, PartialEq)]
pub struct Links {
    pub participant: String,
    pub conversation: String,
}

/// Possible filters for listing participant conversations via the Twilio API
#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
struct ListParams {
    identity: Option<String>,
    address: Option<String>,
}

impl ParticipantConversations<'_> {
    /// [Lists Participant Conversations](https://www.twilio.com/docs/conversations/api/participant-conversation-resource#list-all-of-a-participants-conversations)
    ///
    /// This will eagerly fetch *all* conversations relating to a particular identity or address on the Twilio account.
    ///
    /// Takes optional parameters:
    /// - `identity` - The identity used for the participant (used for participants using the Conversations SDK).
    /// - `address` - Or the address the participant is communicating on. This typically links directly to `messaging_binding.address` of a Conversation.
    pub async fn list(
        &self,
        identity: Option<String>,
        address: Option<String>,
    ) -> Result<Vec<ParticipantConversation>, TwilioError> {
        let params = ListParams { identity, address };

        let mut participant_conversations_page = self
            .client
            .send_request::<ParticipantConversationPage, ListParams>(
                Method::GET,
                "https://conversations.twilio.com/v1/ParticipantConversations",
                Some(&params),
                None,
            )
            .await?;

        let mut results: Vec<ParticipantConversation> =
            participant_conversations_page.conversations;

        while (participant_conversations_page.meta.next_page_url).is_some() {
            participant_conversations_page = self
                .client
                .send_request::<ParticipantConversationPage, ()>(
                    Method::GET,
                    &participant_conversations_page.meta.next_page_url.unwrap(),
                    None,
                    None,
                )
                .await?;

            results.append(&mut participant_conversations_page.conversations);
        }

        Ok(results)
    }
}
