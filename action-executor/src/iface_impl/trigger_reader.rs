use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use async_trait::async_trait;

use gcloud::{auth, pubsub};

use protocol::ActionManifest;

use toolkit::{message::Message, queue::MemoryQueue};

use crate::interfaces::ActionManifestQueueReader;

pub struct PubsubActionManifestQueueReader {
    subscription: pubsub::Subscription<ActionManifest, pubsub::formats::JSON>,
}

impl PubsubActionManifestQueueReader {
    pub async fn new(
        project_id: String,
        authenticator: auth::AuthProvider,
        subscription_id: String,
    ) -> Result<Self> {
        let client = pubsub::Client::new(&project_id, authenticator).await?;
        let subscription = client.subscription(&subscription_id).await?;

        Ok(PubsubActionManifestQueueReader { subscription })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        subscription_id: String,
    ) -> Result<Self> {
        let authenticator = auth::AuthProvider::from_json_file(credentials_file_path)?;
        PubsubActionManifestQueueReader::new(project_id, authenticator, subscription_id).await
    }
}

#[async_trait]
impl ActionManifestQueueReader for PubsubActionManifestQueueReader {
    async fn pull_action_manifest(
        &self,
    ) -> Result<Option<Box<dyn Message<ActionManifest> + Send>>> {
        let msg = self.subscription.pull().await?;
        Ok(msg)
    }
}

pub struct InMemoryActionManifestQueueReader {
    queue: Arc<MemoryQueue>,
}

impl InMemoryActionManifestQueueReader {
    pub fn new(queue: Arc<MemoryQueue>) -> Self {
        Self { queue }
    }
}

#[async_trait]
impl ActionManifestQueueReader for InMemoryActionManifestQueueReader {
    async fn pull_action_manifest(
        &self,
    ) -> Result<Option<Box<dyn Message<ActionManifest> + Send>>> {
        let msg: Option<Box<dyn Message<ActionManifest> + Send>> = self.queue.pull()?;
        Ok(msg)
    }
}
