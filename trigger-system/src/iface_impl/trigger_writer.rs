use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{ensure, Result};

use gcloud::{pub_sub::PubSubClient, AuthProvider};

use crate::interface::{Trigger, TriggerQueueWriter};

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

    pub fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        topic: String,
    ) -> Result<Self> {
        let authenticator = AuthProvider::from_json_file(credentials_file_path)?;
        Ok(PubsubTriggerQueueWriter::new(
            project_id,
            authenticator,
            topic,
        ))
    }
}

impl TriggerQueueWriter for PubsubTriggerQueueWriter {
    fn push_trigger(&self, trigger: Trigger) -> Result<()> {
        self.client.publish(trigger, &self.topic)?;
        Ok(())
    }
}

/// Writes triggers to a directory.
pub struct DirectoryTriggerQueueWriter {
    counter: AtomicU64,
    path: PathBuf,
}

impl DirectoryTriggerQueueWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        ensure!(
            path.as_ref().exists(),
            format!("{:?} does not exist", path.as_ref())
        );

        Ok(Self {
            counter: AtomicU64::new(0),
            path: PathBuf::from(path.as_ref()),
        })
    }
}

impl TriggerQueueWriter for DirectoryTriggerQueueWriter {
    fn push_trigger(&self, trigger: Trigger) -> Result<()> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.path.join(format!("trigger_{}.txt", value));

        let file_handle = fs::File::create(path)?;
        serde_json::to_writer(file_handle, &trigger)?;
        Ok(())
    }
}
