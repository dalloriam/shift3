use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time;

use anyhow::{ensure, Result};

use plugin_core::TriggerPlugin;
use plugin_host::PluginHost;

use protocol::TriggerConfiguration;

use crate::{BoxedCfgLoader, BoxedQueueWriter};

const EXIT_POLL_FREQUENCY: time::Duration = time::Duration::from_millis(100);
const CONFIG_UPDATE_FREQUENCY: time::Duration = time::Duration::from_secs(60 * 5); // Default to 5 min. TODO: Make configurable.

/// The trigger manager is the "main" thread of the trigger system.
pub struct TriggerManager {
    cfg_loader: BoxedCfgLoader,
    queue_writer: BoxedQueueWriter,
    stop_rx: mpsc::Receiver<()>,

    configs: Vec<TriggerConfiguration>,
    last_config_update: time::Instant,

    executors: HashMap<String, Arc<Box<dyn TriggerPlugin>>>,

    plugin_host: Arc<PluginHost>,
}

impl TriggerManager {
    pub fn new(
        stop_rx: mpsc::Receiver<()>,
        cfg_loader: BoxedCfgLoader,
        queue_writer: BoxedQueueWriter,
        plugin_host: Arc<PluginHost>,
    ) -> Result<Self> {
        let configs = cfg_loader.get_all_configurations()?;

        let mut manager = TriggerManager {
            cfg_loader,
            queue_writer,
            stop_rx,

            configs,
            last_config_update: time::Instant::now(),

            executors: HashMap::new(),
            plugin_host,
        };

        manager.refresh_plugins()?;

        Ok(manager)
    }

    fn refresh_plugins(&mut self) -> Result<()> {
        self.executors.clear();
        for trigger_plugin in self.plugin_host.get_trigger_plugins() {
            let trigger_name = String::from(trigger_plugin.get_type());
            self.executors.insert(trigger_name, trigger_plugin.clone());
        }

        Ok(())
    }

    fn execute_trigger(&mut self, cfg: &TriggerConfiguration) -> Result<()> {
        log::debug!("checking trigger {}/{}", &cfg.trigger_type, cfg.id);

        let executor_maybe = self.executors.get_mut(&cfg.trigger_type);

        ensure!(
            executor_maybe.is_some(),
            format!("Unknown trigger type: {}", cfg.trigger_type)
        );

        // unwrap is safe because of ensure()
        for trigger in executor_maybe.unwrap().pull_trigger(cfg)? {
            self.queue_writer.push_trigger(trigger)?;
        }
        Ok(())
    }

    fn check_all_triggers(&mut self) -> Result<()> {
        log::debug!("begin checking all triggers");

        let now = time::Instant::now();
        if now.duration_since(self.last_config_update) > CONFIG_UPDATE_FREQUENCY {
            // Update trigger configs.
            log::info!("refreshing trigger configs");
            self.configs = self.cfg_loader.get_all_configurations()?;
            self.last_config_update = now;
            log::info!("trigger config refresh complete");
        }

        let configs_copy = self.configs.clone();
        for config in configs_copy.into_iter() {
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
