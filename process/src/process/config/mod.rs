use std::path::PathBuf;
use std::sync::Arc;

mod executor;
mod interpreter;
mod trigger;

use anyhow::Result;

use plugin_host::PluginHost;

use serde::{Deserialize, Serialize};

use crate::Service;

#[derive(Deserialize, Serialize)]
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
    pub fn into_instance(self, host: Arc<PluginHost>) -> Result<Service> {
        match self {
            SystemConfiguration::Trigger(cfg) => cfg.into_instance(host.clone()),
            SystemConfiguration::Interpreter(cfg) => cfg.into_instance(host.clone()),
            SystemConfiguration::Executor(cfg) => cfg.into_instance(host.clone()),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::trigger::{
        ConfigReaderConfiguration, QueueWriterConfiguration, TriggerSystemConfiguration,
    };
    use super::SystemConfiguration;

    #[test]
    fn test_into_instance() {
        let cfg = SystemConfiguration::Trigger(TriggerSystemConfiguration {
            config_reader: ConfigReaderConfiguration::File {
                file: PathBuf::from("/var"),
            },
            queue_writer: QueueWriterConfiguration::Directory {
                path: PathBuf::from("/var"),
            },
        });

        cfg.into_instance().unwrap();
    }
}
