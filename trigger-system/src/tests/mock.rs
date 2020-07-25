use std::convert::Infallible;

use crate::interface::{Trigger, TriggerConfigLoader, TriggerConfiguration, TriggerQueueWriter};

pub struct Dummy {}

impl Default for Dummy {
    fn default() -> Self {
        Self {}
    }
}

impl TriggerConfigLoader for Dummy {
    type Error = Infallible;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error> {
        Ok(Vec::new())
    }
}

impl TriggerQueueWriter for Dummy {
    type Error = Infallible;

    fn push_trigger(&self, _trigger: Trigger) -> Result<(), Self::Error> {
        Ok(())
    }
}
