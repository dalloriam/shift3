use anyhow::{Context, Result};

use crate::interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader};

use crate::manager::TriggerManager;

use std::sync::{Arc, Mutex};
use toolkit::thread::{JoinHolder, StoppableThread};

/// The trigger interpreter manages the operations of the trigger service.
/// It manages its own threads and resources.
pub struct TriggerInterpreter {
    handles: Vec<StoppableThread<()>>,
}

impl TriggerInterpreter {
    /// Starts the trigger interpreter
    pub fn start<
        R: 'static + TriggerQueueReader + Send + Clone,
        A: 'static + ActionConfigReader + Send,
        W: 'static + ActionManifestQueueWriter + Send,
    >(
        queue_reader: R,
        cfg_reader: A,
        queue_writer: W,
    ) -> Self {
        log::debug!("begin pulling trigger data");

        let mut interpreter = Self {
            handles: Vec::new(),
        };

        let config = Arc::new(Mutex::new(cfg_reader));
        let writer = Arc::new(Mutex::new(queue_writer));

        for _ in 0..9 {
            let config_copy = config.clone();
            let writer_copy = writer.clone();
            let reader_copy = queue_reader.clone();

            interpreter.handles.push(StoppableThread::spawn(
                move |stop_rx| match TriggerManager::new(
                    stop_rx,
                    reader_copy,
                    config_copy,
                    writer_copy,
                ) {
                    Ok(man) => man.start(),
                    Err(e) => log::error!("failed to start interpreter manager: {:?}", e),
                },
            ));
        }

        log::info!("interpreter started");

        interpreter
    }

    pub fn stop(self) -> Result<()> {
        log::info!("received request to stop");

        // Send stop signal to all threads.
        let join_handles: Result<Vec<JoinHolder<()>>> = self
            .handles
            .into_iter()
            .map(|h| h.stop().context("Failed to stop trigger manager"))
            .collect();

        // Join all threads.
        for join_holder in join_handles?.into_iter() {
            join_holder
                .join()
                .context("Failed to join trigger manager thread")?;
        }

        log::info!("stop complete");

        Ok(())
    }
}
