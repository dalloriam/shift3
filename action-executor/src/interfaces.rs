use protocol::ActionManifest;

pub trait ActionManifestQueueReader {
    type Error;

    fn pull_action_manifest(&self) -> Result<ActionManifest, Self::Error>;
}
