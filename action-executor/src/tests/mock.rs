use std::sync::{Arc, Mutex};

use anyhow::Result;

use async_std::sync::{Arc as AsyncArc, Mutex as AsyncMutex};

use async_trait::async_trait;

use protocol::ActionManifest;

use toolkit::message::{Error as MessageError, Message};

use crate::interfaces::ActionManifestQueueReader;

pub struct DummyMessage {
    manifest: ActionManifest,
    ack_callback_vec: AsyncArc<AsyncMutex<Vec<String>>>,
}

#[async_trait]
impl Message<ActionManifest> for DummyMessage {
    async fn ack(&mut self) -> std::result::Result<(), MessageError> {
        let mut guard = self.ack_callback_vec.lock().await;
        (*guard).push("asdf".into());
        Ok(())
    }

    fn data(&self) -> std::result::Result<ActionManifest, MessageError> {
        Ok(self.manifest.clone())
    }
}

#[derive(Default)]
pub struct Dummy {}

#[async_trait]
impl ActionManifestQueueReader for Dummy {
    async fn pull_action_manifest(
        &self,
    ) -> Result<Option<Box<dyn Message<ActionManifest> + Send>>> {
        Ok(None)
    }
}

#[derive(Default)]
pub struct InMemoryReader {
    pub incoming_queue: Vec<ActionManifest>,
    pub received_acks: AsyncArc<AsyncMutex<Vec<String>>>,
}

impl InMemoryReader {
    async fn internal_ack_count(&self) -> usize {
        let guard = self.received_acks.lock().await;
        guard.len()
    }

    pub fn ack_count(&self) -> usize {
        async_std::task::block_on(self.internal_ack_count())
    }
}

type MultiThreadedReader = Arc<Mutex<InMemoryReader>>;

#[async_trait]
impl ActionManifestQueueReader for MultiThreadedReader {
    async fn pull_action_manifest(
        &self,
    ) -> Result<Option<Box<dyn Message<ActionManifest> + Send>>> {
        let mut guard = self.lock().unwrap();
        let reader = &mut *guard;

        let msg_maybe: Option<Box<dyn Message<ActionManifest> + Send>> =
            match reader.incoming_queue.pop() {
                Some(manifest) => Some(Box::from(DummyMessage {
                    manifest,
                    ack_callback_vec: reader.received_acks.clone(),
                })),
                None => None,
            };
        Ok(msg_maybe)
    }
}
