use std::sync::Arc;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use action_executor::{
    iface_impl::{InMemoryActionManifestQueueReader, PubsubActionManifestQueueReader},
    ActionManifestQueueReader, ExecutorSystem, ExecutorSystemConfig,
};

use crate::{ResourceManager, Service};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ExecutorSystemConfiguration {
    pub queue_reader: QueueReaderConfiguration,
}

impl ExecutorSystemConfiguration {
    pub async fn into_instance(self, resource_manager: Arc<ResourceManager>) -> Result<Service> {
        let queue_reader = self
            .queue_reader
            .into_instance(resource_manager.clone())
            .await?;
        Ok(Box::from(ExecutorSystem::start(ExecutorSystemConfig {
            queue_reader,
            plugin_host: resource_manager.get_plugin_host(),
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
    InMemory {
        topic: String,
    },
}

impl QueueReaderConfiguration {
    async fn into_instance(
        self,
        resource_manager: Arc<ResourceManager>,
    ) -> Result<Box<dyn ActionManifestQueueReader + Send>> {
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
            QueueReaderConfiguration::InMemory { topic } => Box::from(
                InMemoryActionManifestQueueReader::new(resource_manager.get_memory_queue(&topic)?),
            ),
        };

        Ok(b)
    }
}
