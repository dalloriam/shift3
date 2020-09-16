use std::path::PathBuf;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::Service;

use trigger_system::{
    iface_impl::config::{datastore::DatastoreTriggerConfigLoader, file::FileTriggerConfigLoader},
    iface_impl::trigger_writer::{DirectoryTriggerQueueWriter, PubsubTriggerQueueWriter},
    TriggerConfigLoader, TriggerQueueWriter, TriggerSystem,
};

/// Configuration struct of the trigger system.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct TriggerSystemConfiguration {
    pub config_reader: ConfigReaderConfiguration,
    pub queue_writer: QueueWriterConfiguration,
}

impl TriggerSystemConfiguration {
    /// Converts the trigger system configuration to a usable service instance.
    pub fn into_instance(self) -> Result<Service> {
        let cfg_loader = self.config_reader.into_instance()?;
        let queue_writer = self.queue_writer.into_instance()?;
        Ok(Box::from(TriggerSystem::start(cfg_loader, queue_writer)))
    }
}

/// Configuration of the trigger config reader.
///
/// Contains configurations for the various supported config readers (e.g. disk, datastore).
#[derive(Deserialize, Serialize, Debug, PartialEq)]
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
    /// Returns a usable trigger config loader from the configuration struct.
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

#[derive(Deserialize, Serialize, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ConfigReaderConfiguration, QueueWriterConfiguration};

    macro_rules! parse_ok {
        ($t: ident, $(($name:ident, $eq_to:expr),)*) => {
            $(
                #[test]
                fn $name() {
                    const DATA_RAW: &str =
                        include_str!(concat!("test_data/", stringify!($name), ".json"));

                    let deserialized: $t = serde_json::from_str(DATA_RAW).unwrap();
                    assert_eq!(deserialized, $eq_to);

                    // We don't care about whether it failed.
                    match deserialized.into_instance() {
                        Ok(_) => {},
                        Err(_) => {}
                    }
                }
            )*
        };
    }

    macro_rules! parse_fail {
        ($t: ident, $($name:ident,)*) => {
            $(
                #[test]
                fn $name() {
                    const DATA_RAW: &str =
                        include_str!(concat!("test_data/", stringify!($name), ".json"));

                    let deserialized_maybe: Result<$t, serde_json::Error> =
                        serde_json::from_str(DATA_RAW);
                    assert!(deserialized_maybe.is_err());
                }
            )*
        };
    }

    // Validates successful parsing of config reader configurations.
    parse_ok! {
        ConfigReaderConfiguration,

        // File config.
        (file_cfg_valid, ConfigReaderConfiguration::File{file: PathBuf::from("/tmp/yeet.json")}),

        // Datastore config.
        (
            ds_cfg_valid,
            ConfigReaderConfiguration::DataStore{
                project_id: String::from("asdf123"),
                credentials_file_path: String::from("/etc/gcloud.json")
            }
        ),
    }

    // Validates that invalid configs fail, as expected.
    parse_fail! {
        ConfigReaderConfiguration,

        file_cfg_invalid,
        cfg_gibberish,
    }

    parse_ok! {
        QueueWriterConfiguration,
        (
            queue_dir_valid,
            QueueWriterConfiguration::Directory {
                path: PathBuf::from("/tmp/yeet/")
            }
        ),
        (
            queue_pubsub_valid,
            QueueWriterConfiguration::PubSub {
                project_id: String::from("asdf123"),
                credentials_file_path: String::from("/etc/gcloud.json"),
                topic: String::from("somequeue"),
            }
        ),
    }

    parse_fail! {
        QueueWriterConfiguration,

        queue_gibberish,
    }
}
