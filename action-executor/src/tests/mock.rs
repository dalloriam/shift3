use std::sync::{Arc, Mutex};

use anyhow::Result;

use protocol::ActionManifest;

use crate::interfaces::ActionManifestQueueReader;

#[derive(Default)]
pub struct Dummy {}

impl ActionManifestQueueReader for Dummy {
    fn batch_ack(&self, _ack_ids: Vec<String>) -> Result<()> {
        Ok(())
    }

    fn pull_action_manifests(&self) -> Result<Vec<(String, ActionManifest)>> {
        Ok(Vec::new())
    }
}

#[derive(Default)]
pub struct InMemoryReader {
    pub incoming_queue: Vec<ActionManifest>,
    pub received_acks: Vec<String>,
}

type MultiThreadedReader = Arc<Mutex<InMemoryReader>>;

impl ActionManifestQueueReader for MultiThreadedReader {
    fn batch_ack(&self, ack_ids: Vec<String>) -> Result<()> {
        let mut guard = self.lock().unwrap(); // safe because we're in a test.

        for id in ack_ids.into_iter() {
            (*guard).received_acks.push(id);
        }

        Ok(())
    }

    fn pull_action_manifests(&self) -> Result<Vec<(String, ActionManifest)>> {
        let mut guard = self.lock().unwrap();

        let mut results = Vec::new();

        while !(*guard).incoming_queue.is_empty() {
            let v = (*guard).incoming_queue.pop().unwrap(); // Safe because not empty.
            results.push((v.data.clone(), v));
        }

        Ok(results)
    }
}
