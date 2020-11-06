use std::{collections::HashMap, sync::Arc, sync::Mutex};

use anyhow::Error;
use protocol::{ActionManifest, Rule, RuleID, Trigger};

use crate::interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader};

pub struct Dummy {}

impl Default for Dummy {
    fn default() -> Self {
        Self {}
    }
}

impl ActionConfigReader for Dummy {
    fn get_rule(&self, id: RuleID) -> Result<Rule, Error> {
        Ok(Rule {
            id,
            trigger_config_id: 1,
            action_config: String::from(""),
            action_type: String::from(""),
        })
    }
}

impl ActionManifestQueueWriter for Dummy {
    fn push_action_manifest(&self, _: ActionManifest) -> Result<(), Error> {
        Ok(())
    }
}

impl TriggerQueueReader for Dummy {
    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>, Error> {
        Ok(Vec::new())
    }

    fn acknowledge(&self, _: Vec<String>) -> Result<(), Error> {
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn TriggerQueueReader + Send> {
        Box::new(Dummy {})
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

impl ActionConfigReader for InMemoryActionConfigReader {
    fn get_rule(&self, id: RuleID) -> Result<Rule, Error> {
        if let Some(rule) = self.configs.get(&id) {
            return Ok(rule.clone());
        }

        Err(Error::msg("Rule not found"))
    }
}

#[derive(Clone)]
pub struct InMemoryQueueReader {
    pub queue: Arc<Mutex<HashMap<String, Trigger>>>,
}

impl InMemoryQueueReader {
    pub fn new(queue: HashMap<String, Trigger>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(queue)),
        }
    }
}

impl TriggerQueueReader for InMemoryQueueReader {
    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>, Error> {
        let mut guard = self.queue.lock().unwrap(); // We won't get poisoning in a simple test.
        let queue_handle = &mut *guard;

        let mut vec: Vec<(String, Trigger)> = Vec::with_capacity(queue_handle.len());

        for (key, value) in queue_handle.iter() {
            vec.push((key.clone(), value.clone()));
        }

        Ok(vec)
    }

    fn acknowledge(&self, ack_ids: Vec<String>) -> Result<(), Error> {
        let mut guard = self.queue.lock().unwrap(); // We won't get poisoning in a simple test.
        let queue_handle = &mut *guard;

        for id in ack_ids {
            queue_handle.remove(&id);
        }

        Ok(())
    }

    fn box_clone(&self) -> Box<dyn TriggerQueueReader + Send> {
        Box::new(InMemoryQueueReader {
            queue: self.queue.clone(),
        })
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

impl ActionManifestQueueWriter for MultiThreadQueueWriter {
    fn push_action_manifest(&self, action_manifest: ActionManifest) -> Result<(), Error> {
        let mut guard = self.lock().unwrap(); // We won't get poisoning in a simple test.
        let queue_handle = &mut *guard;

        queue_handle.queue.push(action_manifest);

        Ok(())
    }
}
