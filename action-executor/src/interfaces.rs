use anyhow::Result;

use protocol::ActionManifest;

pub trait ActionManifestQueueReader {
    fn pull_action_manifest(&self) -> Result<ActionManifest>;
}
