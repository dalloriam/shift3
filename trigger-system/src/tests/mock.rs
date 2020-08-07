use std::sync::{Arc, Mutex};

use anyhow::Error;

use crate::interface::{Trigger, TriggerConfigLoader, TriggerConfiguration, TriggerQueueWriter};

pub struct Dummy {}

impl Default for Dummy {
    fn default() -> Self {
        Self {}
    }
}

impl TriggerConfigLoader for Dummy {
    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error> {
        Ok(Vec::new())
    }
}

impl TriggerQueueWriter for Dummy {
    fn push_trigger(&self, _trigger: Trigger) -> Result<(), Error> {
        Ok(())
    }
}

pub struct InMemoryConfigLoader {
    configs: Vec<TriggerConfiguration>,
}

impl InMemoryConfigLoader {
    pub fn new(configs: Vec<TriggerConfiguration>) -> Self {
        Self { configs }
    }
}

impl TriggerConfigLoader for InMemoryConfigLoader {
    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error> {
        Ok(self.configs.clone())
    }
}

pub struct InMemoryQueueWriter {
    pub queue: Vec<Trigger>,
}

impl InMemoryQueueWriter {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }
}

type MultiThreadQueueWriter = Arc<Mutex<Box<InMemoryQueueWriter>>>;

impl TriggerQueueWriter for MultiThreadQueueWriter {
    fn push_trigger(&self, trigger: Trigger) -> Result<(), Error> {
        let mut guard = self.lock().unwrap(); // We won't get poisoning in a simple test.
        let queue_handle = &mut *guard;
        queue_handle.queue.push(trigger);
        Ok(())
    }
}
