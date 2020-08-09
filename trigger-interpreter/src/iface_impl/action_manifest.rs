use std::fmt;

use gcloud::{pub_sub::PubSubClient, AuthProvider};
use protocol::ActionManifest;

use crate::interface::ActionManifestQueueWriter;

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
}

impl ActionManifestQueueWriter for PubSubActionManifestWriter {
    type Error = Error;

    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<(), Self::Error> {
        let result = self
            .client
            .publish(manifest, self.topic_id.as_str())
            .map_err(|ds| Error::PubSubError(format!("{:?}", ds)))?;

        Ok(result)
    }
}
