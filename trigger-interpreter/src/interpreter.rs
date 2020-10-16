use anyhow::{Context, Error, Result};
use toolkit::{thread::StoppableThread, Stop};

use crate::{manager::TriggerManager, BoxedCfgReader, BoxedQueueReader, BoxedQueueWriter};

/// The trigger interpreter manages the operations of the trigger service.
/// It manages its own thread and resources.
pub struct TriggerInterpreter {
    handle: StoppableThread<()>,
}

impl TriggerInterpreter {
    /// Starts the trigger interpreter
    pub fn start(
        queue_reader: BoxedQueueReader,
        cfg_reader: BoxedCfgReader,
        queue_writer: BoxedQueueWriter,
    ) -> Self {
        log::debug!("begin pulling trigger data");

        let interpreter = Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                match TriggerManager::new(stop_rx, queue_reader, cfg_reader, queue_writer) {
                    Ok(man) => man.start(),
                    Err(e) => log::error!("failed to start interpreter manager: {:?}", e),
                }
            }),
        };

        log::info!("interpreter started");

        interpreter
    }

    /// Called by Stop. Used to enable terminating
    /// the system without boxing it first.
    pub fn terminate(self) -> Result<()> {
        log::info!("received request to stop");

        self.handle
            .stop()
            .context("Failed to stop trigger interpreter manager")?
            .join()
            .context("Failed to join trigger manager interpreter thread")?;

        log::info!("stop complete");

        Ok(())
    }
}

impl Stop for TriggerInterpreter {
    type Error = Error;

    fn stop(self: Box<Self>) -> Result<()> {
        self.terminate()
    }
}
