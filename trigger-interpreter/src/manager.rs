use std::sync::{mpsc, Arc, Mutex};

use anyhow::Result;

use protocol::{ActionManifest, Trigger};

use crate::{templating::render_template, BoxedCfgReader, BoxedQueueReader, BoxedQueueWriter};

/// The interpreter manager is the "main" thread of the trigger interpreter.
pub struct TriggerManager {
    queue_reader: BoxedQueueReader,
    cfg_reader: Arc<Mutex<BoxedCfgReader>>,
    queue_writer: Arc<Mutex<BoxedQueueWriter>>,
    stop_rx: mpsc::Receiver<()>,
}

impl TriggerManager {
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        queue_reader: BoxedQueueReader,
        cfg_reader: Arc<Mutex<BoxedCfgReader>>,
        queue_writer: Arc<Mutex<BoxedQueueWriter>>,
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

        log::debug!("fetching the rule ({}) by its id", trigger.rule);
        let rule = cfg_reader_ref.get_rule(trigger.rule)?;
        log::debug!("rule fetched {:?}", rule);

        log::debug!("rendering the template from the action configuration");
        let action_config = render_template(rule.action_config, trigger.data)?;
        log::debug!("template rendered: {:?}", action_config);

        let action_manifest = ActionManifest {
            rule: trigger.rule,
            action_type: rule.action_type,
            data: action_config,
        };

        let mut queue_writer_guard = self.queue_writer.lock().unwrap();
        let queue_writer_ref = &mut (*queue_writer_guard);

        log::debug!("pushing the action manifest");
        queue_writer_ref.push_action_manifest(action_manifest)?;

        Ok(())
    }

    fn pull_trigger(&self) -> Result<()> {
        log::debug!("begin pulling trigger data");

        loop {
            let triggers = self.queue_reader.pull_trigger()?;
            log::debug!("number of messages pulled ({:?})", triggers.len());

            for (id, trigger) in triggers {
                self.interpret_trigger(trigger)?;

                log::debug!("acknowledging the message ({})", id);

                let mut ids = Vec::new();
                ids.push(id);

                self.queue_reader.acknowlege(ids)?;
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
