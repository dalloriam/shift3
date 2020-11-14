use anyhow::Result;

use async_trait::async_trait;

use protocol::{ActionManifest, Rule, RuleID, Trigger};

/// Trait describing an object capable of pulling a trigger from a queue.
pub trait TriggerQueueReader {
    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>>;

    fn acknowlege(&self, ack_ids: Vec<String>) -> Result<()>;

    fn box_clone(&self) -> Box<dyn TriggerQueueReader + Send>;
}

/// Trait describing an object capable of pulling a trigger from a queue.
#[async_trait]
pub trait ActionConfigReader {
    async fn get_rule(&self, id: RuleID) -> Result<Rule>;
}

/// Trait describing an object capable of pushing an action manifest to a queue.
#[async_trait]
pub trait ActionManifestQueueWriter {
    async fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()>;
}
