use protocol::{ActionManifest, Rule, RuleID, Trigger};

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait TriggerQueueReader {
    type Error: std::error::Error + Send + Sync;

    fn pull_trigger(&self) -> Result<Trigger, Self::Error>;
}

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait ActionConfigReader {
    type Error: std::error::Error + Send + Sync;

    fn get_rule(&self, id: RuleID) -> Result<Rule, Self::Error>;
}

/// Trait describing an object capable of pushing an action manifest to a queue.
pub trait ActionManifestQueueWriter {
    type Error: std::error::Error + Send + Sync;

    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<(), Self::Error>;
}
