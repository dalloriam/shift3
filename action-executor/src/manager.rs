use std::collections::HashMap;
use std::sync::{mpsc, Arc};

use anyhow::Result;

use plugin_core::{ActionPlugin, Error as PluginError};
use plugin_host::PluginHost;

use crate::BoxedQueueReader;

pub struct ExecutorManager {
    manifest_reader: BoxedQueueReader,

    stop_rx: mpsc::Receiver<()>,

    executors: HashMap<String, Arc<Box<dyn ActionPlugin>>>,

    plugin_host: Arc<PluginHost>,
}

impl<'a> ExecutorManager {
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        manifest_reader: BoxedQueueReader,
        plugin_host: Arc<PluginHost>,
    ) -> Result<Self> {
        let mut hsh: HashMap<String, Arc<Box<dyn ActionPlugin>>> = HashMap::new();
        for action_plugin in plugin_host.get_action_plugins() {
            let action_name = String::from(action_plugin.get_type());
            hsh.insert(action_name, action_plugin.clone());
        }

        let mut manager = ExecutorManager {
            manifest_reader,
            stop_rx,
            executors: hsh,
            plugin_host,
        };

        Ok(manager)
    }

    pub fn load_executors(&'a mut self) -> Result<()> {
        Ok(())
    }

    fn pull_cycle(&mut self) -> Result<()> {
        let mut ack_ids = Vec::with_capacity(10); // TODO: Match batch size.
        let mut res: Result<(), PluginError> = Ok(());

        for (ack_id, action_manifest) in self.manifest_reader.pull_action_manifests()? {
            log::debug!("got manifest: {:?}", action_manifest);

            if let Some(ex) = self.executors.get(&action_manifest.action_type) {
                res = ex.execute_action(action_manifest);
                if res.is_err() {
                    break;
                }
            } else {
                log::warn!("unknown action type: {:?}", &action_manifest.action_type);
            }

            ack_ids.push(ack_id);
        }

        if !ack_ids.is_empty() {
            self.manifest_reader.batch_ack(ack_ids)?;
        }

        res?;
        Ok(())
    }

    pub fn start(&mut self) {
        log::debug!("executor loop running");

        loop {
            if let Err(e) = self.pull_cycle() {
                log::error!("{:?}", e);
            }

            if self.stop_rx.try_recv().is_ok() {
                log::debug!("executor stopping");
                break;
            }
        }
    }
}
