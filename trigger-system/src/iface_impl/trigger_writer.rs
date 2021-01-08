use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use async_std::sync::Mutex;

use anyhow::{anyhow, ensure, Result};

use async_trait::async_trait;

use gcloud::AuthProvider;

use google_cloud::pubsub;

use toolkit::queue::MemoryQueue;

use crate::interface::{Trigger, TriggerQueueWriter};

pub struct PubsubTriggerQueueWriter {
    topic: Mutex<pubsub::Topic>,
}

impl PubsubTriggerQueueWriter {
    pub async fn new(
        project_id: String,
        authenticator: AuthProvider,
        topic: String,
    ) -> Result<Self> {
        let mut client =
            pubsub::Client::from_credentials(&project_id, authenticator.into()).await?;
        let topic = client
            .topic(&topic)
            .await?
            .ok_or_else(|| anyhow!("topic doesn't exist"))?;

        Ok(Self {
            topic: Mutex::from(topic),
        })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        topic: String,
    ) -> Result<Self> {
        let authenticator = AuthProvider::from_json_file(credentials_file_path)?;
        PubsubTriggerQueueWriter::new(project_id, authenticator, topic).await
    }
}

#[async_trait]
impl TriggerQueueWriter for PubsubTriggerQueueWriter {
    async fn push_trigger(&self, trigger: Trigger) -> Result<()> {
        let mut guard = self.topic.lock().await;
        let data = serde_json::to_vec(&trigger)?;
        (*guard).publish(data).await?;
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

#[async_trait]
impl TriggerQueueWriter for DirectoryTriggerQueueWriter {
    async fn push_trigger(&self, trigger: Trigger) -> Result<()> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.path.join(format!("trigger_{}.txt", value));

        let file_handle = fs::File::create(path)?;
        serde_json::to_writer(file_handle, &trigger)?;
        Ok(())
    }
}

pub struct InMemoryTriggerQueueWriter {
    queue: Arc<MemoryQueue>,
}

impl InMemoryTriggerQueueWriter {
    pub fn new(queue: Arc<MemoryQueue>) -> Self {
        InMemoryTriggerQueueWriter { queue }
    }
}

#[async_trait]
impl TriggerQueueWriter for InMemoryTriggerQueueWriter {
    async fn push_trigger(&self, trigger: Trigger) -> Result<()> {
        self.queue.publish(trigger)?;
        Ok(())
    }
}
