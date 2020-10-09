use anyhow::{Context, Error, Result};

use toolkit::{thread::StoppableThread, Stop};

use crate::manager::ExecutorManager;
use crate::BoxedQueueReader;

pub struct ExecutorSystem {
    handle: StoppableThread<()>,
}

impl ExecutorSystem {
    pub fn start(queue_reader: BoxedQueueReader) -> Self {
        log::debug!("starting system");

        let sys = Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                match ExecutorManager::new(stop_rx, queue_reader) {
                    Ok(mut e) => e.start(),
                    Err(err) => log::error!("failed to start the manager: {:?}", err),
                }
            }),
        };

        log::info!("system started");

        sys
    }

    pub fn terminate(self) -> Result<()> {
        log::info!("received request to stop");

        self.handle
            .stop()
            .context("Failed to stop executor: ")?
            .join()
            .context("Failed to join executor thread")?;

        log::info!("stop complete");

        Ok(())
    }
}

impl Stop for ExecutorSystem {
    type Error = Error;

    fn stop(self: Box<Self>) -> Result<()> {
        self.terminate()
    }
}
