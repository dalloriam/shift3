use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use plugin_host::PluginHost;

use toolkit::db::sled::SledStore;
use toolkit::queue::MemoryQueue;

use crate::Configuration;

/// The resource manager is the struct that manages and holds
/// objects that can be requested by the various systems as they boot, such as the
/// plugin host, in-memory datastructures and queues, etc.
#[derive(Default)]
pub struct ResourceManager {
    plugin_host: Arc<PluginHost>,

    queues: Mutex<HashMap<String, Arc<MemoryQueue>>>,

    sleds: Mutex<HashMap<PathBuf, Arc<SledStore>>>,
}

impl ResourceManager {
    pub fn new(config: &Configuration) -> Result<ResourceManager> {
        Ok(ResourceManager {
            plugin_host: Arc::from(PluginHost::initialize(&config.plugin_paths)?),
            queues: Mutex::new(HashMap::<String, Arc<MemoryQueue>>::new()),
            sleds: Mutex::new(HashMap::<PathBuf, Arc<SledStore>>::new()),
        })
    }

    pub fn get_plugin_host(&self) -> Arc<PluginHost> {
        self.plugin_host.clone()
    }

    pub fn get_memory_queue(&self, queue_name: &str) -> Result<Arc<MemoryQueue>> {
        let mut queues_guard = self.queues.lock().map_err(|e| anyhow!(e.to_string()))?;
        let queues = &mut *queues_guard;

        if let Some(q) = queues.get(queue_name) {
            return Ok(q.clone());
        }

        // Create a new queue.
        let queue = Arc::from(MemoryQueue::new());

        // TODO: Add persist path.

        queues.insert(String::from(queue_name), queue.clone());

        Ok(queue)
    }

    pub fn get_embedded_store<P: AsRef<Path>>(&self, path: P) -> Result<Arc<SledStore>> {
        let mut sleds_guard = self.sleds.lock().map_err(|e| anyhow!(e.to_string()))?;
        let sleds = &mut *sleds_guard;

        if let Some(s) = sleds.get(path.as_ref()) {
            return Ok(s.clone());
        }

        let sled = Arc::from(SledStore::new(path.as_ref())?);

        sleds.insert(PathBuf::from(path.as_ref()), sled.clone());

        Ok(sled)
    }
}
