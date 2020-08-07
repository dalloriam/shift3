use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Writes triggers to a directory.
pub struct DirectoryTriggerQueueWriter {
    counter: AtomicU64,
    path: PathBuf,
}

impl TriggerQueueWriter for DirectoryTriggerQueueWriter {
    type Error = io::Error;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.path.join(format!("trigger_{}.txt", value));

        let file_handle = fs::File::create(path)?;
        serde_json::to_writer(file_handle, &trigger)?;
        Ok(())
    }
}
