use std::sync::mpsc;

use anyhow::{Error, Result};

use protocol::{ActionManifest, Trigger};

use crate::{templating::render_template, BoxedCfgReader, BoxedQueueReader, BoxedQueueWriter};

/// The interpreter manager is the "main" thread of the trigger interpreter.
pub struct TriggerManager {
    queue_reader: BoxedQueueReader,
    cfg_reader: BoxedCfgReader,
    queue_writer: BoxedQueueWriter,
    stop_rx: mpsc::Receiver<()>,
}

impl TriggerManager {
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        queue_reader: BoxedQueueReader,
        cfg_reader: BoxedCfgReader,
        queue_writer: BoxedQueueWriter,
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

        log::debug!("fetching the rule ({}) by its id", trigger.rule);
        // Get action configuration associated with the trigger's rule.
        let rule = self.cfg_reader.get_rule(trigger.rule)?;
        log::debug!("rule fetched {:?}", rule);

        log::debug!("rendering the template from the action configuration");
        let action_config = render_template(rule.action_config, trigger.data)?;
        log::debug!("template rendered: {:?}", action_config);

        let action_manifest = ActionManifest {
            rule: trigger.rule,
            action_type: rule.action_type,
            data: action_config,
        };

        log::debug!("pushing the action manifest");
        self.queue_writer.push_action_manifest(action_manifest)?;

        Ok(())
    }

    fn pull_trigger(&self) -> Result<()> {
        let triggers = self.queue_reader.pull_trigger()?;
        log::debug!("number of messages pulled ({:?})", triggers.len());

        // TODO: Use same capacity as the batch size used during queue reading
        let mut ack_ids = Vec::with_capacity(10);
        let mut result: Result<(), Error> = Ok(());

        for (id, trigger) in triggers {
            if let Err(err) = self.interpret_trigger(trigger) {
                result = Err(err);
                break;
            }

            ack_ids.push(id);
        }

        if !ack_ids.is_empty() {
            self.queue_reader.acknowlege(ack_ids)?;
        }

        result
    }

    pub fn start(&self) {
        log::debug!("begin pulling trigger data");

        loop {
            if let Err(e) = self.pull_trigger() {
                log::error!("{:?}", e);
            }

            if self.stop_rx.try_recv().is_ok() {
                break;
            }
        }
    }
}
