use serde::{Deserialize, Serialize};

use action_executor::{iface_impl::PubsubActionManifestQueueReader, ActionManifestQueueReader};

use crate::Service;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ExecutorSystemConfiguration {}

impl ExecutorSystemConfiguration {
    pub fn into_instance(self) -> Result<Service> {
        Ok(Box::from(Exeuc))
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
            } => Ok(Box::from()),
        }
    }
}
