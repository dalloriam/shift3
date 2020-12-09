use anyhow::Result;

use async_trait::async_trait;

use protocol::{ActionManifest, Rule, Trigger};

use toolkit::message::Message;

/// Trait describing an object capable of pulling a trigger from a queue.
#[async_trait]
pub trait TriggerQueueReader {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>>;
}

/// Trait describing an object capable of pulling a trigger from a queue.
#[async_trait]
pub trait ActionConfigReader {
    async fn get_rule(&self, id: &str) -> Result<Rule>;
}

/// Trait describing an object capable of pushing an action manifest to a queue.
#[async_trait]
pub trait ActionManifestQueueWriter {
    async fn push_action_manifest(&self, manifest: ActionManifest) -> Result<()>;
}
