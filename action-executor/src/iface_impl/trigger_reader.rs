use std::path::Path;

use anyhow::{anyhow, Result};

use async_std::sync::Mutex;

use async_trait::async_trait;

use gcloud::AuthProvider;

use google_cloud::pubsub;

use protocol::ActionManifest;

use serde::de::DeserializeOwned;

use crate::interfaces::{ActionManifestQueueReader, Message};

pub struct JSONPubsubMessage {
    message: pubsub::Message,
}

#[async_trait]
impl<T> Message<T> for JSONPubsubMessage
where
    T: DeserializeOwned + 'static,
{
    async fn ack(&mut self) -> Result<()> {
        self.message.ack().await?;
        Ok(())
    }

    fn data(&self) -> Result<T> {
        let deserialized = serde_json::from_slice(self.message.data())?;
        Ok(deserialized)
    }
}

impl From<pubsub::Message> for JSONPubsubMessage {
    fn from(m: pubsub::Message) -> JSONPubsubMessage {
        JSONPubsubMessage { message: m }
    }
}

pub struct PubsubActionManifestQueueReader {
    subscription: Mutex<pubsub::Subscription>,
}

impl PubsubActionManifestQueueReader {
    pub async fn new(
        project_id: String,
        authenticator: AuthProvider,
        subscription_id: String,
    ) -> Result<Self> {
        let mut client =
            pubsub::Client::from_credentials(&project_id, authenticator.into()).await?;

        let subscription = client
            .subscription(&subscription_id)
            .await?
            .ok_or_else(|| anyhow!("subscription doesn't exist"))?;

        Ok(PubsubActionManifestQueueReader {
            subscription: Mutex::from(subscription),
        })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        subscription_id: String,
    ) -> Result<Self> {
        let authenticator: AuthProvider = AuthProvider::from_json_file(credentials_file_path)?;
        PubsubActionManifestQueueReader::new(project_id, authenticator, subscription_id).await
    }
}

#[async_trait]
impl ActionManifestQueueReader for PubsubActionManifestQueueReader {
    async fn pull_action_manifest(
        &self,
    ) -> Result<Option<Box<dyn Message<ActionManifest> + Send>>> {
        let mut subscription_guard = self.subscription.lock().await;
        match subscription_guard.receive().await {
            Some(m) => Ok(Some(Box::from(JSONPubsubMessage::from(m)))),
            None => Ok(None),
        }
    }
}
