use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::Service;

use trigger_interpreter::{
    iface_impl::DatastoreActionConfigLoader, iface_impl::PubSubActionManifestWriter,
    iface_impl::PubSubTriggerReader, ActionConfigReader, ActionManifestQueueWriter,
    TriggerInterpreter, TriggerQueueReader,
};

/// Configuration struct of the trigger interpreter.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct TriggerInterpreterConfiguration {
    pub config_reader: ConfigReaderConfiguration,
    pub queue_reader: QueueReaderConfiguration,
    pub queue_writer: QueueWriterConfiguration,
}

impl TriggerInterpreterConfiguration {
    /// Converts the trigger interpreter configuration to a usable service instance.
    pub fn into_instance(self) -> Result<Service> {
        let cfg_reader = self.config_reader.into_instance()?;
        let queue_writer = self.queue_writer.into_instance()?;
        let queue_reader = self.queue_reader.into_instance()?;

        Ok(Box::from(TriggerInterpreter::start(
            queue_reader,
            cfg_reader,
            queue_writer,
        )))
    }
}

/// Configuration of the action config reader.
///
/// Contains configurations for the various supported config readers (e.g. disk, datastore).
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")] // ConfigReaderConfiguration::Disk{blah: blah} => {type: Disk, blah: blah}
pub enum ConfigReaderConfiguration {
    DataStore {
        project_id: String,
        credentials_file_path: String,
    },
}

impl ConfigReaderConfiguration {
    /// Returns a usable action config reader from the configuration struct.
    pub fn into_instance(self) -> Result<Box<dyn ActionConfigReader + Send>> {
        match self {
            ConfigReaderConfiguration::DataStore {
                project_id,
                credentials_file_path,
            } => Ok(Box::from(DatastoreActionConfigLoader::from_credentials(
                project_id,
                credentials_file_path,
            )?)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum QueueWriterConfiguration {
    PubSub {
        project_id: String,
        credentials_file_path: String,
        topic: String,
    },
}

impl QueueWriterConfiguration {
    pub fn into_instance(self) -> Result<Box<dyn ActionManifestQueueWriter + Send>> {
        match self {
            QueueWriterConfiguration::PubSub {
                project_id,
                credentials_file_path,
                topic,
            } => Ok(Box::from(PubSubActionManifestWriter::from_credentials(
                project_id,
                credentials_file_path,
                topic,
            )?)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum QueueReaderConfiguration {
    PubSub {
        project_id: String,
        credentials_file_path: String,
        subscription: String,
    },
}

impl QueueReaderConfiguration {
    pub fn into_instance(self) -> Result<Box<dyn TriggerQueueReader + Send>> {
        match self {
            QueueReaderConfiguration::PubSub {
                project_id,
                credentials_file_path,
                subscription,
            } => Ok(Box::from(PubSubTriggerReader::from_credentials(
                project_id,
                credentials_file_path,
                subscription,
            )?)),
        }
    }
}
