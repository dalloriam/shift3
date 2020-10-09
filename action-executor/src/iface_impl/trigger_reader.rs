use std::path::Path;

use anyhow::{ensure, Result};

use gcloud::{pub_sub::PubSubClient, AuthProvider};

use protocol::ActionManifest;

use crate::interfaces::ActionManifestQueueReader;

pub struct PubsubActionManifestQueueReader {
    client: PubSubClient,
    subscription_id: String,
}

impl PubsubActionManifestQueueReader {
    pub fn new(project_id: String, authenticator: AuthProvider, subscription_id: String) -> Self {
        PubsubActionManifestQueueReader {
            client: PubSubClient::new(project_id.clone(), authenticator),
            subscription_id,
        }
    }

    pub fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        subscription_id: String,
    ) -> Result<Self> {
        let authenticator: AuthProvider = AuthProvider::from_json_file(credentials_file_path)?;
        Ok(PubsubActionManifestQueueReader::new(
            project_id,
            authenticator,
            subscription_id,
        ))
    }
}

impl ActionManifestQueueReader for PubsubActionManifestQueueReader {
    fn batch_ack(&self, ack_ids: Vec<String>) -> Result<()> {
        self.client.acknowledge(ack_ids, &self.subscription_id)?;
        Ok(())
    }

    fn pull_action_manifests(&self) -> Result<Vec<(String, ActionManifest)>> {
        let results = self.client.pull(self.subscription_id.as_ref(), 10)?;
        Ok(results)
    }
}
