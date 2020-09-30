use anyhow::Result;

use protocol::ActionManifest;

pub trait ActionManifestQueueReader {
    fn pull_action_manifests(&self) -> Result<Vec<(String, ActionManifest)>>;
}
