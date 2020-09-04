use std::sync::mpsc;

use anyhow::Result;

use crate::BoxedQueueReader;

pub struct ExecutorManager {
    manifest_reader: BoxedQueueReader,

    stop_rx: mpsc::Receiver<()>,
}

impl ExecutorManager {
    pub fn new(stop_rx: mpsc::Receiver<()>, manifest_reader: BoxedQueueReader) -> Result<Self> {
        Ok(ExecutorManager {
            manifest_reader,
            stop_rx,
        })
    }

    pub fn start(&mut self) {}
}
