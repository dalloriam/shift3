use std::sync::{mpsc, Arc, Mutex};
use std::time;

use anyhow::Result;

use protocol::{ActionManifest, Trigger};

use crate::{
    interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader},
    templating::render_template,
};

const EXIT_POLL_FREQUENCY: time::Duration = time::Duration::from_millis(100);

/// TODO: Comment this!
pub struct TriggerManager<R, A, W>
where
    R: 'static + TriggerQueueReader,
    A: 'static + ActionConfigReader,
    W: 'static + ActionManifestQueueWriter,
{
    queue_reader: Arc<Mutex<R>>,
    cfg_reader: Arc<Mutex<A>>,
    queue_writer: Arc<Mutex<W>>,
    stop_rx: mpsc::Receiver<()>,
}

impl<R, A, W> TriggerManager<R, A, W>
where
    R: 'static + TriggerQueueReader + Send,
    A: 'static + ActionConfigReader + Send,
    W: 'static + ActionManifestQueueWriter + Send,
{
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        queue_reader: Arc<Mutex<R>>,
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

        let mut queue_reader_guard = self.queue_reader.lock().unwrap();
        let queue_reader_ref = &mut (*queue_reader_guard);
        let trigger = queue_reader_ref.pull_trigger()?;

        self.interpret_trigger(trigger)?;

        Ok(())
    }

    pub fn start(&self) {
        loop {
            if let Err(e) = self.pull_trigger() {
                log::error!("{:?}", e);
            }
        }
    }
}
