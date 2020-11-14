use std::sync::{Arc, Mutex};

use anyhow::Error;

use async_trait::async_trait;

use crate::interface::{Trigger, TriggerConfigLoader, TriggerConfiguration, TriggerQueueWriter};

pub struct Dummy {}

impl Default for Dummy {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl TriggerConfigLoader for Dummy {
    async fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl TriggerQueueWriter for Dummy {
    async fn push_trigger(&self, _trigger: Trigger) -> Result<(), Error> {
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

#[async_trait]
impl TriggerConfigLoader for InMemoryConfigLoader {
    async fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error> {
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

type MultiThreadQueueWriter = Arc<Mutex<InMemoryQueueWriter>>;

#[async_trait]
impl TriggerQueueWriter for MultiThreadQueueWriter {
    async fn push_trigger(&self, trigger: Trigger) -> Result<(), Error> {
        let mut guard = self.lock().unwrap(); // We won't get poisoning in a simple test.
        let queue_handle = &mut *guard;
        queue_handle.queue.push(trigger);
        Ok(())
    }
}
