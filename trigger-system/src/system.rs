use anyhow::{Context, Result};

use toolkit::thread::StoppableThread;

use crate::manager::TriggerManager;
use crate::{BoxedCfgLoader, BoxedQueueWriter};

/// The trigger system manages the operation of the trigger service.
/// It manages its own threads and resources.
pub struct TriggerSystem {
    handle: StoppableThread<()>,
}

impl TriggerSystem {
    /// Creates a new trigger system.
    pub fn start(cfg_loader: BoxedCfgLoader, queue_writer: BoxedQueueWriter) -> Self {
        log::debug!("starting system");

        let sys = Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                match TriggerManager::new(stop_rx, cfg_loader, queue_writer) {
                    Ok(mut man) => man.start(),
                    Err(e) => log::error!("failed to start manager: {:?}", e),
                }
            }),
        };

        log::info!("system started");

        sys
    }

    pub fn stop(mut self) -> Result<()> {
        log::info!("received request to stop");

        self.handle
            .join()
            .context("Failed to stop trigger manager")?;

        log::info!("stop complete");

        Ok(())
    }
}
