use std::sync::Arc;

use anyhow::{Context, Error, Result};

use plugin_host::PluginHost;

use toolkit::{thread::StoppableThread, Stop};

use crate::manager::TriggerManager;
use crate::{BoxedCfgLoader, BoxedQueueWriter};

pub struct TriggerSystemConfig {
    pub config_loader: BoxedCfgLoader,
    pub queue_writer: BoxedQueueWriter,
    pub plugin_host: Arc<PluginHost>,
}

/// The trigger system manages the operation of the trigger service.
/// It manages its own threads and resources.
pub struct TriggerSystem {
    handle: StoppableThread<()>,
}

impl TriggerSystem {
    /// Creates a new trigger system.
    pub fn start(cfg: TriggerSystemConfig) -> Self {
        log::debug!("starting system");

        let sys = Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                match TriggerManager::new(
                    stop_rx,
                    cfg.config_loader,
                    cfg.queue_writer,
                    cfg.plugin_host,
                ) {
                    Ok(mut man) => man.start(),
                    Err(e) => log::error!("failed to start manager: {:?}", e),
                }
            }),
        };

        log::info!("system started");

        sys
    }

    /// Called by Stop. Used to enable terminating
    /// the system without boxing it first.
    pub fn terminate(self) -> Result<()> {
        log::info!("received request to stop");

        self.handle
            .stop()
            .context("Failed to stop trigger manager")?
            .join()
            .context("Failed to join trigger manager thread")?;

        log::info!("stop complete");

        Ok(())
    }
}

impl Stop for TriggerSystem {
    type Error = Error;

    fn stop(self: Box<Self>) -> Result<()> {
        self.terminate()
    }
}
