pub use protocol::{Trigger, TriggerConfiguration};

pub trait TriggerConfigLoader {
    type Error: std::error::Error + Send + Sync;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error>;
}

pub trait TriggerQueueWriter {
    type Error: std::error::Error + Send + Sync;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error>;
}
