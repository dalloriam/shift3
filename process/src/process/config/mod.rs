use std::path::PathBuf;
use std::sync::Arc;

mod executor;
mod interpreter;
mod trigger;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::{ResourceManager, Service};

#[derive(Default, Deserialize, Serialize)]
pub struct Configuration {
    pub plugin_paths: Vec<PathBuf>,
    pub systems: Vec<SystemConfiguration>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SystemConfiguration {
    Trigger(trigger::TriggerSystemConfiguration),
    Interpreter(interpreter::TriggerInterpreterConfiguration),
    Executor(executor::ExecutorSystemConfiguration),
}

impl SystemConfiguration {
    pub async fn into_instance(self, manager: Arc<ResourceManager>) -> Result<Service> {
        match self {
            SystemConfiguration::Trigger(cfg) => cfg.into_instance(manager).await,
            SystemConfiguration::Interpreter(cfg) => cfg.into_instance(manager).await,
            SystemConfiguration::Executor(cfg) => cfg.into_instance(manager).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::trigger::{
        ConfigReaderConfiguration, QueueWriterConfiguration, TriggerSystemConfiguration,
    };
    use super::{ResourceManager, SystemConfiguration};
    use crate::Configuration;

    #[tokio::test]
    async fn test_into_instance() {
        let cfg = SystemConfiguration::Trigger(TriggerSystemConfiguration {
            config_reader: ConfigReaderConfiguration::File {
                file: PathBuf::from("/var"),
            },
            queue_writer: QueueWriterConfiguration::Directory {
                path: PathBuf::from("/var"),
            },
        });

        cfg.into_instance(Arc::new(
            ResourceManager::new(&Configuration::default()).unwrap(),
        ))
        .await
        .unwrap();
    }
}
