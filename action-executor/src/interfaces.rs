use anyhow::Result;

use protocol::ActionManifest;

pub trait ActionManifestQueueReader {
    fn batch_ack(&self, ack_ids: Vec<String>) -> Result<()>;
    fn pull_action_manifests(&self) -> Result<Vec<(String, ActionManifest)>>;
}
