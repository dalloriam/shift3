use std::sync::Arc;

use anyhow::Result;

use plugin_host::PluginHost;

use crate::{Configuration, Service};

/// A single process node.
pub struct Node {
    _plugin_host: Arc<PluginHost>, // TODO: This handle will  be needed for the persistence API.
    services: Vec<Service>,
}

impl Node {
    fn initialize_plugin_host(config: &Configuration) -> Result<Arc<PluginHost>> {
        let host = PluginHost::initialize(&config.plugin_paths)?;
        Ok(Arc::new(host))
    }

    pub fn start(config: Configuration) -> Result<Node> {
        // Initialize plugin host first and foremost.
        // We need to do this first, because the services might want to pull some things from
        // the plugins when initializing.
        let plugin_host = Node::initialize_plugin_host(&config)?;

        let services: Result<Vec<Service>> = config
            .systems
            .into_iter()
            .map(|sys_cfg| sys_cfg.into_instance(plugin_host.clone()))
            .collect();

        Ok(Node {
            _plugin_host: plugin_host,
            services: services?,
        })
    }

    pub fn stop(self) -> Result<()> {
        for svc in self.services.into_iter() {
            svc.stop()?;
        }
        Ok(())
    }
}
