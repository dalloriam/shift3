use std::fmt;

use gcloud::{
    pub_sub::{PubSubClient, PubSubError},
    AuthProvider,
};

use crate::interface::{Trigger, TriggerQueueWriter};

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

impl From<PubSubError> for Error {
    fn from(e: PubSubError) -> Self {
        Error::PubSubError(format!("{}", e))
    }
}

pub struct PubsubTriggerQueueWriter {
    client: PubSubClient,
    topic: String,
}

impl PubsubTriggerQueueWriter {
    pub fn new(project_id: String, authenticator: AuthProvider, topic: String) -> Self {
        Self {
            client: PubSubClient::new(project_id, authenticator),
            topic,
        }
    }
}

impl TriggerQueueWriter for PubsubTriggerQueueWriter {
    type Error = Error;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error> {
        self.client.publish(trigger, &self.topic)?;
        Ok(())
    }
}
