use std::path::PathBuf;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::Service;

use trigger_system::{
    iface_impl::config::{datastore::DatastoreTriggerConfigLoader, file::FileTriggerConfigLoader},
    iface_impl::trigger_writer::{DirectoryTriggerQueueWriter, PubsubTriggerQueueWriter},
    TriggerConfigLoader, TriggerQueueWriter, TriggerSystem,
};

#[derive(Deserialize, Serialize)]
pub struct TriggerSystemConfiguration {
    config_reader: ConfigReaderConfiguration,
    queue_writer: QueueWriterConfiguration,
}

impl TriggerSystemConfiguration {
    pub fn into_instance(self) -> Result<Service> {
        let cfg_loader = self.config_reader.into_instance()?;
        let queue_writer = self.queue_writer.into_instance()?;
        Ok(Box::from(TriggerSystem::start(cfg_loader, queue_writer)))
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")] // ConfigReaderConfiguration::Disk{blah: blah} => {type: Disk, blah: blah}
pub enum ConfigReaderConfiguration {
    DataStore {
        project_id: String,
        credentials_file_path: String,
    },
    File {
        file: PathBuf,
    },
}

impl ConfigReaderConfiguration {
    /// Returns a usable trigger config loader from the configuration.
    pub fn into_instance(self) -> Result<Box<dyn TriggerConfigLoader + Send>> {
        match self {
            ConfigReaderConfiguration::File { file } => {
                Ok(Box::from(FileTriggerConfigLoader::new(file)?))
            }
            ConfigReaderConfiguration::DataStore {
                project_id,
                credentials_file_path,
            } => Ok(Box::from(DatastoreTriggerConfigLoader::from_credentials(
                project_id,
                credentials_file_path,
            )?)),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum QueueWriterConfiguration {
    Directory {
        path: PathBuf,
    },
    PubSub {
        project_id: String,
        credentials_file_path: String,
        topic: String,
    },
}

impl QueueWriterConfiguration {
    pub fn into_instance(self) -> Result<Box<dyn TriggerQueueWriter + Send>> {
        match self {
            QueueWriterConfiguration::Directory { path } => {
                Ok(Box::from(DirectoryTriggerQueueWriter::new(path)?))
            }
            QueueWriterConfiguration::PubSub {
                project_id,
                credentials_file_path,
                topic,
            } => Ok(Box::from(PubsubTriggerQueueWriter::from_credentials(
                project_id,
                credentials_file_path,
                topic,
            )?)),
        }
    }
}
