use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{ensure, Error, Result};
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

impl ActionManifestQueueWriter for FileActionManifestWriter {
    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.path.join(format!("action_manifest_{}.txt", value));

        let file_handle = fs::File::create(path)?;
        serde_json::to_writer(file_handle, &manifest)?;

        Ok(())
    }
}
