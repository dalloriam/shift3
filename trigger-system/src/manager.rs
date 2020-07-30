use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time;

use anyhow::{ensure, Result};

use protocol::TriggerConfiguration;

use crate::exec::{load_executors, ExecutorObj};
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

    executors: HashMap<String, ExecutorObj>,
}

impl<T, Q> TriggerManager<T, Q>
where
    T: 'static + TriggerConfigLoader,
    Q: 'static + TriggerQueueWriter,
{
    pub fn new(stop_rx: mpsc::Receiver<()>, cfg_loader: T, queue_writer: Q) -> Result<Self> {
        Ok(TriggerManager {
            cfg_loader,
            queue_writer,
            stop_rx,

            executors: load_executors()?,
        })
    }

    fn execute_trigger(&mut self, cfg: &TriggerConfiguration) -> Result<()> {
        log::debug!("checking trigger {}/{}", &cfg.trigger_type, cfg.id);

        let executor_maybe = self.executors.get_mut(&cfg.trigger_type);

        ensure!(
            executor_maybe.is_some(),
            format!("Unknown trigger type: {}", cfg.trigger_type)
        );

        // unwrap is safe because of ensure()
        for trigger in executor_maybe.unwrap().execute(&cfg)? {
            self.queue_writer.push_trigger(trigger)?;
        }
        Ok(())
    }

    fn check_all_triggers(&mut self) -> Result<()> {
        log::debug!("begin checking all triggers");

        // TODO: Don't fetch every configs every 100ms... :)
        for config in self.cfg_loader.get_all_configurations()? {
            self.execute_trigger(&config)?;
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
