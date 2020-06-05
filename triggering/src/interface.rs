pub use crate::{Trigger, TriggerConfiguration};

pub trait TriggerConfigLoader {
    type Error;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error>;
}

pub trait TriggerQueueWriter {
    type Error;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error>;
}
