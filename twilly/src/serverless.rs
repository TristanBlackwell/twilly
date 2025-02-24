/*!

Contains Twilio Serverless related functionality.

*/
pub mod environments;
pub mod services;

use crate::Client;

use self::services::{Service, Services};

/// Holds Serverless related functions accessible
/// on the client.
pub struct Serverless<'a> {
    pub client: &'a Client,
}

impl<'a> Serverless<'a> {
    /// Actions relating to a known Function Service.
    ///
    /// Takes in the SID of the Service to perform actions against.
    pub fn service<'b: 'a>(&self, sid: &'b str) -> Service {
        Service {
            client: self.client,
            sid,
        }
    }

    /// General Function Service actions.
    pub fn services(&self) -> Services {
        Services {
            client: self.client,
        }
    }
}
