use anyhow::Result;

use async_trait::async_trait;

use protocol::ActionManifest;

use serde::de::DeserializeOwned;

#[async_trait]
pub trait Message<T>
where
    T: DeserializeOwned,
{
    async fn ack(&mut self) -> Result<()>;
    fn data(&self) -> Result<T>;
}

#[async_trait]
pub trait ActionManifestQueueReader {
    async fn pull_action_manifest(&self)
        -> Result<Option<Box<dyn Message<ActionManifest> + Send>>>;
}
