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
    project_id: String,
    authenticator: AuthProvider,
}

impl Clone for PubSubTriggerReader {
    fn clone(&self) -> Self {
        PubSubTriggerReader {
            client: PubSubClient::new(self.project_id.clone(), self.authenticator.clone()),
            subscription_id: self.subscription_id.clone(),
            project_id: self.project_id.clone(),
            authenticator: self.authenticator.clone(),
        }
    }
}

impl PubSubTriggerReader {
    pub fn new(project_id: String, authenticator: AuthProvider, subscription_id: String) -> Self {
        PubSubTriggerReader {
            client: PubSubClient::new(project_id.clone(), authenticator.clone()),
            subscription_id,
            project_id,
            authenticator,
        }
    }
}

impl TriggerQueueReader for PubSubTriggerReader {
    type Error = Error;

    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>, Self::Error> {
        let result = self
            .client
            .pull(self.subscription_id.as_str(), 10) // TODO: Us config instead of hardcoded value
            .map_err(|ds| Error::PubSubError(format!("{:?}", ds)))?;

        Ok(result)
    }

    fn acknowlege(&self, ack_ids: Vec<String>) -> Result<(), Self::Error> {
        self.client
            .acknowledge(ack_ids, self.subscription_id.as_str())
            .map_err(|ds| Error::PubSubError(format!("{:?}", ds)))?;

        Ok(())
    }
}
