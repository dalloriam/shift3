use std::sync::Arc;

use anyhow::Result;

use crate::{Configuration, ResourceManager, Service};

/// A single process node.
pub struct Node {
    _resource_manager: Arc<ResourceManager>, // TODO: This handle will  be needed for the persistence API.
    services: Vec<Service>,
}

impl Node {
    fn initialize_resource_manager(config: &Configuration) -> Result<Arc<ResourceManager>> {
        let manager = ResourceManager::new(config)?;
        Ok(Arc::new(manager))
    }

    pub async fn start(config: Configuration) -> Result<Node> {
        // Initialize resource manager first and foremost.
        // We need to do this first, because the services might want to pull some things from
        // it when initializing.
        let resource_manager = Node::initialize_resource_manager(&config)?;

        let mut services: Vec<Service> = Vec::new();
        for system_config in config.systems.into_iter() {
            services.push(
                system_config
                    .into_instance(resource_manager.clone())
                    .await?,
            );
        }

        Ok(Node {
            _resource_manager: resource_manager,
            services,
        })
    }

    pub fn stop(self) -> Result<()> {
        for svc in self.services.into_iter() {
            svc.stop()?;
        }
        Ok(())
    }
}
