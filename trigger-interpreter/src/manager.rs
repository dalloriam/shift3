use std::sync::mpsc;

use anyhow::Result;

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

    async fn interpret_trigger(&self, trigger: Trigger) -> Result<()> {
        log::debug!("begin interpreting the trigger data");

        log::debug!("fetching the rule ({}) by its id", trigger.rule);
        // Get action configuration associated with the trigger's rule.
        let rule = self.cfg_reader.get_rule(trigger.rule).await?;
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
        self.queue_writer
            .push_action_manifest(action_manifest)
            .await?;

        Ok(())
    }

    async fn pull_trigger(&self) -> Result<()> {
        if let Some(mut message) = self.queue_reader.pull_trigger().await? {
            let trigger = message.data()?;
            self.interpret_trigger(trigger).await?;
            message.ack().await?;
        }

        Ok(())
    }

    async fn asynchronous_main_loop(&self) {
        log::debug!("begin pulling trigger data");

        loop {
            if let Err(e) = self.pull_trigger().await {
                log::error!("{:?}", e);
            }

            if self.stop_rx.try_recv().is_ok() {
                break;
            }
        }
    }

    pub fn start(&self) {
        async_std::task::block_on(self.asynchronous_main_loop());
    }
}
