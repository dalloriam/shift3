use anyhow::Result;

use crate::{Configuration, Service};

/// A single process node.
pub struct Node {
    services: Vec<Service>,
}

impl Node {
    pub fn start(config: Configuration) -> Result<Node> {
        let services: Result<Vec<Service>> = config
            .systems
            .into_iter()
            .map(|sys_cfg| sys_cfg.into_instance())
            .collect();

        Ok(Node {
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
