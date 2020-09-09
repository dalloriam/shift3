use anyhow::Result;
use std::sync::{mpsc, Arc, Mutex};

use protocol::{ActionManifest, Trigger};

use crate::{
    interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader},
    templating::render_template,
};

/// The interpreter manager is the "main" thread of the trigger interpreter.
pub struct TriggerManager<R, A, W>
where
    R: 'static + TriggerQueueReader,
    A: 'static + ActionConfigReader,
    W: 'static + ActionManifestQueueWriter,
{
    queue_reader: R,
    cfg_reader: Arc<Mutex<A>>,
    queue_writer: Arc<Mutex<W>>,
    stop_rx: mpsc::Receiver<()>,
}

impl<R, A, W> TriggerManager<R, A, W>
where
    R: 'static + TriggerQueueReader + Send + Clone,
    A: 'static + ActionConfigReader + Send,
    W: 'static + ActionManifestQueueWriter + Send,
{
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        queue_reader: R,
        cfg_reader: Arc<Mutex<A>>,
        queue_writer: Arc<Mutex<W>>,
    ) -> Result<Self> {
        Ok(Self {
            queue_reader,
            cfg_reader,
            queue_writer,
            stop_rx,
        })
    }

    fn interpret_trigger(&self, trigger: Trigger) -> Result<()> {
        log::debug!("begin interpreting the trigger data");

        // Get action configuration associated with the trigger's rule.
        let mut cfg_reader_guard = self.cfg_reader.lock().unwrap();
        let cfg_reader_ref = &mut (*cfg_reader_guard);
        let rule = cfg_reader_ref.get_rule(trigger.rule)?;

        let action_config = render_template(rule.action_config, trigger.data)?;

        let action_manifest = ActionManifest {
            rule: trigger.rule,
            action_type: rule.action_type,
            data: action_config,
        };

        let mut queue_writer_guard = self.queue_writer.lock().unwrap();
        let queue_writer_ref = &mut (*queue_writer_guard);
        queue_writer_ref.push_action_manifest(action_manifest)?;

        Ok(())
    }

    fn pull_trigger(&self) -> Result<()> {
        log::debug!("begin pulling trigger data");

        loop {
            let triggers = self.queue_reader.pull_trigger()?;

            for trigger in triggers {
                self.interpret_trigger(trigger)?;
            }

            if self.stop_rx.try_recv().is_ok() {
                break;
            }
        }

        Ok(())
    }

    pub fn start(&self) {
        if let Err(e) = self.pull_trigger() {
            log::error!("{:?}", e);
        }
    }
}
