use protocol::{ActionManifest, Trigger};

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait TriggerQueueReader {
    type Error;

    fn pull_trigger(&self) -> Result<Trigger, Self::Error>;
}

/// Trait describing an object capable of pushing an action manifest to a queue.
pub trait ActionManifestQueueWriter {
    type Error;

    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<(), Self::Error>;
}
