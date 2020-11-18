use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use plugin_host::PluginHost;

use toolkit::queue::MemoryQueue;

use crate::Configuration;

/// The resource manager is the struct that manages and holds
/// objects that can be requested by the various systems as they boot, such as the
/// plugin host, in-memory datastructures and queues, etc.
pub struct ResourceManager {
    plugin_host: Arc<PluginHost>,

    queues: Mutex<HashMap<String, Arc<MemoryQueue>>>,
}

impl ResourceManager {
    pub fn new(config: &Configuration) -> Result<ResourceManager> {
        Ok(ResourceManager {
            plugin_host: Arc::from(PluginHost::initialize(&config.plugin_paths)?),
            queues: Mutex::new(HashMap::<String, Arc<MemoryQueue>>::new()),
        })
    }

    pub fn get_plugin_host(&self) -> Arc<PluginHost> {
        return self.plugin_host.clone();
    }

    pub fn get_memory_queue<P: AsRef<Path>>(&self, queue_name: &str) -> Result<Arc<MemoryQueue>> {
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
}
