use std::collections::HashMap;
use std::sync::mpsc;

use anyhow::Result;

use crate::{
    exec::{load_executors, ExecutorObj},
    BoxedQueueReader,
};

pub struct ExecutorManager {
    manifest_reader: BoxedQueueReader,

    stop_rx: mpsc::Receiver<()>,

    executors: HashMap<String, ExecutorObj>,
}

impl ExecutorManager {
    pub fn new(stop_rx: mpsc::Receiver<()>, manifest_reader: BoxedQueueReader) -> Result<Self> {
        Ok(ExecutorManager {
            manifest_reader,
            stop_rx,
            executors: load_executors()?,
        })
    }

    fn pull_cycle(&mut self) -> Result<()> {
        let mut ack_ids = Vec::with_capacity(10); // TODO: Match batch size.
        let mut res: Result<()> = Ok(());

        for (ack_id, action_manifest) in self.manifest_reader.pull_action_manifests()? {
            log::debug!("got manifest: {:?}", action_manifest);

            if let Some(ex) = self.executors.get(&action_manifest.action_type) {
                res = ex.execute(action_manifest);
                if res.is_err() {
                    break;
                }
            } else {
                log::warn!("unknown action type: {:?}", &action_manifest.action_type);
            }

            ack_ids.push(ack_id);
        }

        if !ack_ids.is_empty() {
            self.manifest_reader.batch_ack(ack_ids)?;
        }

        res
    }

    pub fn start(&mut self) {
        log::debug!("executor loop running");
        loop {
            if let Err(e) = self.pull_cycle() {
                log::error!("{:?}", e);
            }

            if self.stop_rx.try_recv().is_ok() {
                log::debug!("executor stopping");
                break;
            }
        }
    }
}
