/*!

Contains Twilio Sync related functionality.

*/
pub mod documents;
pub mod maps;
pub mod services;

use crate::Client;

use self::services::{Service, Services};

/// Holds Sync related functions accessible
/// on the client.
pub struct Sync<'a> {
    pub client: &'a Client,
}

impl<'a> Sync<'a> {
    /// Functions relating to a known Sync Service.
    ///
    /// Takes in the SID of the Sync Service to perform actions against.
    pub fn service<'b: 'a>(&'a self, sid: &'b str) -> Service {
        Service {
            client: self.client,
            sid,
        }
    }

    /// General Sync Service functions.
    pub fn services<'b>(&'a self) -> Services {
        Services {
            client: self.client,
        }
    }
}
