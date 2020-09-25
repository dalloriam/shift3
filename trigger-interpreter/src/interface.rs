use anyhow::Error;

use protocol::{ActionManifest, Rule, RuleID, Trigger};

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait TriggerQueueReader {
    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>, Error>;

    fn acknowlege(&self, ack_ids: Vec<String>) -> Result<(), Error>;

    fn box_clone(&self) -> Box<dyn TriggerQueueReader + Send>;
}

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait ActionConfigReader {
    fn get_rule(&self, id: RuleID) -> Result<Rule, Error>;
}

/// Trait describing an object capable of pushing an action manifest to a queue.
pub trait ActionManifestQueueWriter {
    fn push_action_manifest(&self, manifest: ActionManifest) -> Result<(), Error>;
}
