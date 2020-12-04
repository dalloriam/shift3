use std::sync::Arc;

use anyhow::Result;

use plugin_host::PluginHost;

use serde::{Deserialize, Serialize};

use action_executor::{
    iface_impl::PubsubActionManifestQueueReader, ActionManifestQueueReader, ExecutorSystem,
    ExecutorSystemConfig,
};

use crate::Service;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ExecutorSystemConfiguration {
    pub queue_reader: QueueReaderConfiguration,
}

impl ExecutorSystemConfiguration {
    pub async fn into_instance(self, plugin_host: Arc<PluginHost>) -> Result<Service> {
        let queue_reader = self.queue_reader.into_instance().await?;
        Ok(Box::from(ExecutorSystem::start(ExecutorSystemConfig {
            queue_reader,
            plugin_host,
        })))
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum QueueReaderConfiguration {
    PubSub {
        project_id: String,
        credentials_file_path: String,
        subscription: String,
    },
}

impl QueueReaderConfiguration {
    async fn into_instance(self) -> Result<Box<dyn ActionManifestQueueReader + Send>> {
        let b: Box<dyn ActionManifestQueueReader + Send> = match self {
            QueueReaderConfiguration::PubSub {
                project_id,
                credentials_file_path,
                subscription,
            } => Box::from(
                PubsubActionManifestQueueReader::from_credentials(
                    project_id,
                    credentials_file_path,
                    subscription,
                )
                .await?,
            ),
        };

        Ok(b)
    }
}
