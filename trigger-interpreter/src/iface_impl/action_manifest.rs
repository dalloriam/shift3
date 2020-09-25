use std::path::Path;

use anyhow::{Error, Result};
use gcloud::{pub_sub::PubSubClient, AuthProvider};
use protocol::ActionManifest;

use crate::interface::ActionManifestQueueWriter;

pub struct PubSubActionManifestWriter {
    client: PubSubClient,
    topic_id: String,
}

impl PubSubActionManifestWriter {
    pub fn new(project_id: String, authenticator: AuthProvider, topic_id: String) -> Self {
        Self {
            client: PubSubClient::new(project_id, authenticator),
            topic_id,
        }
    }

    pub fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        topic: String,
    ) -> Result<Self> {
        let authenticator = AuthProvider::from_json_file(credentials_file_path)?;
        Ok(Self::new(project_id, authenticator, topic))
    }
}

impl ActionManifestQueueWriter for PubSubActionManifestWriter {
    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()> {
        let result = self
            .client
            .publish(manifest, self.topic_id.as_str())
            .map_err(|ds| Error::msg(format!("{:?}", ds)))?;

        Ok(result)
    }
}
