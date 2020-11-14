use anyhow::Result;

use async_trait::async_trait;

use protocol::ActionManifest;

use toolkit::message::Message;

#[async_trait]
pub trait ActionManifestQueueReader {
    async fn pull_action_manifest(&self)
        -> Result<Option<Box<dyn Message<ActionManifest> + Send>>>;
}
