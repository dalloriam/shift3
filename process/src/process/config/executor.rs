use anyhow::Result;

use serde::{Deserialize, Serialize};

use action_executor::{
    iface_impl::PubsubActionManifestQueueReader, ActionManifestQueueReader, ExecutorSystem,
};

use crate::Service;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ExecutorSystemConfiguration {
    pub queue_reader: QueueReaderConfiguration,
}

impl ExecutorSystemConfiguration {
    pub fn into_instance(self) -> Result<Service> {
        let queue_reader = self.queue_reader.into_instance()?;

        Ok(Box::from(ExecutorSystem::start(queue_reader)))
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
    fn into_instance(self) -> Result<Box<dyn ActionManifestQueueReader + Send>> {
        match self {
            QueueReaderConfiguration::PubSub {
                project_id,
                credentials_file_path,
                subscription,
            } => Ok(Box::from(
                PubsubActionManifestQueueReader::from_credentials(
                    project_id,
                    credentials_file_path,
                    subscription,
                )?,
            )),
        }
    }
}
