use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use anyhow::Result;

use plugin_core::ActionPlugin;
use plugin_host::PluginHost;

use crate::BoxedQueueReader;

pub struct ExecutorManager {
    manifest_reader: BoxedQueueReader,

    stop_rx: mpsc::Receiver<()>,

    executors: HashMap<String, Arc<Box<dyn ActionPlugin>>>,

    plugin_host: Arc<PluginHost>,
}

impl ExecutorManager {
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        manifest_reader: BoxedQueueReader,
        plugin_host: Arc<PluginHost>,
    ) -> Result<Self> {
        let mut manager = ExecutorManager {
            manifest_reader,
            stop_rx,
            executors: HashMap::new(),
            plugin_host,
        };

        manager.refresh_plugins()?;

        Ok(manager)
    }

    fn refresh_plugins(&mut self) -> Result<()> {
        self.executors.clear();
        for action_plugin in self.plugin_host.get_action_plugins() {
            let action_name = String::from(action_plugin.get_type());
            self.executors.insert(action_name, action_plugin.clone());
        }

        Ok(())
    }

    async fn pull_cycle(&mut self) -> Result<()> {
        if let Some(mut msg) = self.manifest_reader.pull_action_manifest().await? {
            // Deserialize message.
            let action_manifest = msg.data()?;

            log::debug!("got manifest: {:?}", action_manifest);

            if let Some(ex) = self.executors.get(&action_manifest.action_type) {
                ex.execute_action(action_manifest)?;
            } else {
                log::warn!("unknown action type: {:?}", &action_manifest.action_type);
            }

            msg.ack().await?;
        }

        Ok(())
    }

    #[tokio::main]
    pub async fn start(&mut self) {
        log::debug!("executor loop running");
        loop {
            if let Err(e) = self.pull_cycle().await {
                log::error!("{:?}", e);
            }

            // TODO: Make this configurable
            thread::sleep(Duration::from_millis(100));

            if self.stop_rx.try_recv().is_ok() {
                log::debug!("executor stopping");
                break;
            }
        }
    }
}
