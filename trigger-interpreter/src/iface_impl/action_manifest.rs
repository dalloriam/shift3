use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use anyhow::{ensure, Result};

use async_trait::async_trait;

use gcloud::{auth, pubsub};

use protocol::ActionManifest;

use toolkit::queue::MemoryQueue;

use crate::interface::ActionManifestQueueWriter;

pub struct PubSubActionManifestWriter {
    topic: pubsub::Topic,
}

impl PubSubActionManifestWriter {
    pub async fn new(
        project_id: String,
        authenticator: auth::AuthProvider,
        topic_id: String,
    ) -> Result<Self> {
        let client = pubsub::Client::new(&project_id, authenticator).await?;
        let topic = client.topic(&topic_id).await?;
        Ok(Self { topic })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        topic: String,
    ) -> Result<Self> {
        let authenticator = auth::AuthProvider::from_json_file(credentials_file_path)?;
        Self::new(project_id, authenticator, topic).await
    }
}

#[async_trait]
impl ActionManifestQueueWriter for PubSubActionManifestWriter {
    async fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()> {
        self.topic.publish(manifest).await?;
        Ok(())
    }
}

/// Writes action manifests to a directory.
pub struct FileActionManifestWriter {
    counter: AtomicU64,
    path: PathBuf,
}

impl FileActionManifestWriter {
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
impl ActionManifestQueueWriter for FileActionManifestWriter {
    async fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.path.join(format!("action_manifest_{}.txt", value));

        let file_handle = fs::File::create(path)?;
        serde_json::to_writer(file_handle, &manifest)?;

        Ok(())
    }
}

pub struct InMemoryActionManifestQueueWriter {
    queue: Arc<MemoryQueue>,
}

impl InMemoryActionManifestQueueWriter {
    pub fn new(queue: Arc<MemoryQueue>) -> Self {
        Self { queue }
    }
}

#[async_trait]
impl ActionManifestQueueWriter for InMemoryActionManifestQueueWriter {
    async fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()> {
        self.queue.publish(manifest)?;
        Ok(())
    }
}
