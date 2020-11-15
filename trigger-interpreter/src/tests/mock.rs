use std::{collections::HashMap, sync::Arc, sync::Mutex};

use anyhow::{anyhow, Result};

use async_std::sync::{Arc as AsyncArc, Mutex as AsyncMutex};

use async_trait::async_trait;

use protocol::{ActionManifest, Rule, RuleID, Trigger};

use toolkit::message::{Error as MessageError, Message};

use crate::interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader};

pub struct Dummy {}

impl Default for Dummy {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl ActionConfigReader for Dummy {
    async fn get_rule(&self, id: RuleID) -> Result<Rule> {
        Ok(Rule {
            id,
            trigger_config_id: 1,
            action_config: String::from(""),
            action_type: String::from(""),
        })
    }
}

#[async_trait]
impl ActionManifestQueueWriter for Dummy {
    async fn push_action_manifest(&self, _: ActionManifest) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl TriggerQueueReader for Dummy {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        Ok(None)
    }
}

pub struct InMemoryActionConfigReader {
    configs: HashMap<RuleID, Rule>,
}

impl InMemoryActionConfigReader {
    pub fn new(configs: HashMap<RuleID, Rule>) -> Self {
        Self { configs }
    }
}

#[async_trait]
impl ActionConfigReader for InMemoryActionConfigReader {
    async fn get_rule(&self, id: RuleID) -> Result<Rule> {
        if let Some(rule) = self.configs.get(&id) {
            return Ok(rule.clone());
        }

        Err(anyhow!("Rule not found"))
    }
}

pub struct MockMessage {
    trigger: Trigger,
    ack_set: AsyncArc<AsyncMutex<Vec<String>>>,
}

#[async_trait]
impl Message<Trigger> for MockMessage {
    async fn ack(&mut self) -> std::result::Result<(), MessageError> {
        let mut vec_guard = self.ack_set.lock().await;
        vec_guard.push(self.trigger.trigger_type.clone());
        Ok(())
    }

    fn data(&self) -> std::result::Result<Trigger, MessageError> {
        Ok(self.trigger.clone())
    }
}

#[derive(Clone)]
pub struct InMemoryQueueReader {
    pub queue: Vec<Trigger>,
    pub ack_set: AsyncArc<AsyncMutex<Vec<String>>>,
}

impl InMemoryQueueReader {
    pub fn new(queue: Vec<Trigger>) -> Self {
        Self {
            queue,
            ack_set: Default::default(),
        }
    }

    async fn internal_ack_count(&self) -> usize {
        let guard = self.ack_set.lock().await;
        (*guard).len()
    }

    pub fn ack_count(&self) -> usize {
        async_std::task::block_on(self.internal_ack_count())
    }
}

#[async_trait]
impl TriggerQueueReader for Arc<Mutex<InMemoryQueueReader>> {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        let mut guard = self.lock().unwrap();
        let reader_handle = &mut *guard;

        let msg_maybe: Option<Box<dyn Message<Trigger> + Send>> = match reader_handle.queue.pop() {
            Some(trigger) => Some(Box::from(MockMessage {
                trigger,
                ack_set: reader_handle.ack_set.clone(),
            })),
            None => None,
        };

        Ok(msg_maybe)
    }
}

pub struct InMemoryQueueWriter {
    pub queue: Vec<ActionManifest>,
}

impl InMemoryQueueWriter {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }
}

type MultiThreadQueueWriter = Arc<Mutex<InMemoryQueueWriter>>;

#[async_trait]
impl ActionManifestQueueWriter for MultiThreadQueueWriter {
    async fn push_action_manifest(&self, action_manifest: ActionManifest) -> Result<()> {
        let mut guard = self.lock().unwrap(); // We won't get poisoning in a simple test.
        let queue_handle = &mut *guard;

        queue_handle.queue.push(action_manifest);

        Ok(())
    }
}
