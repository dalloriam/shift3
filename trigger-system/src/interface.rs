use anyhow::Error;

pub use protocol::{Trigger, TriggerConfiguration};

pub trait TriggerConfigLoader {
    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error>;
}

pub trait TriggerQueueWriter {
    fn push_trigger(&self, trigger: Trigger) -> Result<(), Error>;
}
