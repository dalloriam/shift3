use std::sync::mpsc;
use std::thread;
use std::time;

use anyhow::Result;

use protocol::TriggerConfiguration;

use crate::interface::{TriggerConfigLoader, TriggerQueueWriter};

const EXIT_POLL_FREQUENCY: time::Duration = time::Duration::from_millis(100);

/// The trigger manager is the "main" thread of the trigger system.
pub struct TriggerManager<T, Q>
where
    T: 'static + TriggerConfigLoader,
    Q: 'static + TriggerQueueWriter,
{
    cfg_loader: T,
    queue_writer: Q,
    stop_rx: mpsc::Receiver<()>,
}

impl<T, Q> TriggerManager<T, Q>
where
    T: 'static + TriggerConfigLoader,
    Q: 'static + TriggerQueueWriter,
{
    pub fn new(stop_rx: mpsc::Receiver<()>, cfg_loader: T, queue_writer: Q) -> Self {
        TriggerManager {
            cfg_loader,
            queue_writer,
            stop_rx,
        }
    }

    fn execute_trigger(&mut self, cfg: &TriggerConfiguration) {
        log::debug!("checking trigger {}/{}", cfg.trigger_type.clone(), cfg.id);
    }

    fn check_all_triggers(&mut self) -> Result<()> {
        log::info!("begin checking all triggers");

        for config in self.cfg_loader.get_all_configurations()? {
            self.execute_trigger(&config)
        }

        Ok(())
    }

    pub fn start(&mut self) {
        loop {
            if self.stop_rx.try_recv().is_ok() {
                break;
            }

            if let Err(e) = self.check_all_triggers() {
                log::error!("{:?}", e);
            }

            thread::sleep(EXIT_POLL_FREQUENCY);
        }
    }
}
