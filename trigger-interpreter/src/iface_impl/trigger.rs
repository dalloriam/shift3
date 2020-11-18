use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{ensure, Result};

use async_trait::async_trait;

use gcloud::{auth, pubsub};

use glob::glob;

use protocol::Trigger;

use serde::de::DeserializeOwned;

use toolkit::message::{Error as MessageError, Message};
use toolkit::queue::MemoryQueue;

use crate::interface::TriggerQueueReader;

pub struct PubSubTriggerReader {
    subscription: pubsub::Subscription<Trigger, pubsub::formats::JSON>,
}

impl PubSubTriggerReader {
    pub async fn new(
        project_id: String,
        authenticator: auth::AuthProvider,
        subscription_id: String,
    ) -> Result<Self> {
        let client = pubsub::Client::new(&project_id, authenticator).await?;
        let subscription = client.subscription(&subscription_id).await?;
        Ok(PubSubTriggerReader { subscription })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        subscription: String,
    ) -> Result<Self> {
        let authenticator = auth::AuthProvider::from_json_file(credentials_file_path)?;
        Self::new(project_id, authenticator, subscription).await
    }
}

#[async_trait]
impl TriggerQueueReader for PubSubTriggerReader {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        let msg = self.subscription.pull().await?;
        Ok(msg)
    }
}

/// Reads triggers from a directory.
pub struct FileTriggerQueueReader {
    path: PathBuf,
}

impl FileTriggerQueueReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        ensure!(
            path.as_ref().exists(),
            format!("{:?} does not exist", path.as_ref())
        );

        Ok(Self {
            path: PathBuf::from(path.as_ref()),
        })
    }
}

#[async_trait]
impl TriggerQueueReader for FileTriggerQueueReader {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        let entries: Vec<PathBuf> = glob(&format!(
            "{}/trigger_*.txt",
            self.path.to_string_lossy().as_ref()
        ))?
        .filter_map(|x| x.ok())
        .collect();

        match entries.first() {
            Some(path) => Ok(Some(Box::from(FileQueueMessage { path: path.clone() }))),
            None => Ok(None),
        }
    }
}

struct FileQueueMessage {
    path: PathBuf,
}

#[async_trait]
impl<T> Message<T> for FileQueueMessage
where
    T: DeserializeOwned + 'static,
{
    async fn ack(&mut self) -> std::result::Result<(), MessageError> {
        Ok(())
    }

    fn data(&self) -> std::result::Result<T, MessageError> {
        let file = fs::File::open(&self.path).map_err(|e| MessageError::DeserializeError {
            message: e.to_string(),
        })?;

        let deserialized =
            serde_json::from_reader(file).map_err(|e| MessageError::DeserializeError {
                message: e.to_string(),
            })?;

        Ok(deserialized)
    }
}

pub struct InMemoryTriggerQueueReader {
    queue: Arc<MemoryQueue>,
}

impl InMemoryTriggerQueueReader {
    pub fn new(queue: Arc<MemoryQueue>) -> Self {
        Self { queue }
    }
}

#[async_trait]
impl TriggerQueueReader for InMemoryTriggerQueueReader {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        let msg: Option<Box<dyn Message<Trigger> + Send>> = self.queue.pull()?;
        Ok(msg)
    }
}
