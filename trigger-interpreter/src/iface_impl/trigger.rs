use std::fmt;

use gcloud::{pub_sub::PubSubClient, AuthProvider};
use protocol::Trigger;

use crate::interface::TriggerQueueReader;

#[derive(Debug)]
pub enum Error {
    PubSubError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub struct PubSubTriggerReader {
    client: PubSubClient,
    subscription_id: String,
}

impl PubSubTriggerReader {
    pub fn new(project_id: String, authenticator: AuthProvider, subscription_id: String) -> Self {
        Self {
            client: PubSubClient::new(project_id, authenticator),
            subscription_id,
        }
    }
}

impl TriggerQueueReader for PubSubTriggerReader {
    type Error = Error;

    fn pull_trigger(&self) -> Result<Trigger, Self::Error> {
        let result = self
            .client
            .pull(self.subscription_id.as_str())
            .map_err(|ds| Error::PubSubError(format!("{:?}", ds)))?;

        Ok(result)
    }
}