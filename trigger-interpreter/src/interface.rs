use protocol::{ActionConfiguration, ActionManifest, RuleID, Trigger};

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait TriggerQueueReader {
    type Error;

    fn pull_trigger(&self) -> Result<Trigger, Self::Error>;
}

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait ActionConfigReader {
    type Error;

    fn get_action_config(&self, id: RuleID) -> Result<ActionConfiguration, Self::Error>;
}

/// Trait describing an object capable of pushing an action manifest to a queue.
pub trait ActionManifestQueueWriter {
    type Error;

    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<(), Self::Error>;
}
