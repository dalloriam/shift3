use anyhow::Error;

use async_trait::async_trait;

pub use protocol::{Trigger, TriggerConfiguration};

#[async_trait]
pub trait TriggerConfigLoader {
    async fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error>;
}

#[async_trait]
pub trait TriggerQueueWriter {
    async fn push_trigger(&self, trigger: Trigger) -> Result<(), Error>;
}
