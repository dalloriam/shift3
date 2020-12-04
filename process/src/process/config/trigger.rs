use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use plugin_host::PluginHost;

use serde::{Deserialize, Serialize};

use crate::Service;

use trigger_system::{
    iface_impl::config::{datastore::DatastoreTriggerConfigLoader, file::FileTriggerConfigLoader},
    iface_impl::trigger_writer::{DirectoryTriggerQueueWriter, PubsubTriggerQueueWriter},
    TriggerConfigLoader, TriggerQueueWriter, TriggerSystem, TriggerSystemConfig,
};

/// Configuration struct of the trigger system.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct TriggerSystemConfiguration {
    pub config_reader: ConfigReaderConfiguration,
    pub queue_writer: QueueWriterConfiguration,
}

impl TriggerSystemConfiguration {
    /// Converts the trigger system configuration to a usable service instance.
    pub async fn into_instance(self, plugin_host: Arc<PluginHost>) -> Result<Service> {
        let config_loader = self.config_reader.into_instance().await?;
        let queue_writer = self.queue_writer.into_instance().await?;
        Ok(Box::from(TriggerSystem::start(TriggerSystemConfig {
            config_loader,
            queue_writer,
            plugin_host,
        })))
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
    pub async fn into_instance(self) -> Result<Box<dyn TriggerConfigLoader + Send>> {
        let r: Box<dyn TriggerConfigLoader + Send> = match self {
            ConfigReaderConfiguration::File { file } => {
                Box::from(FileTriggerConfigLoader::new(file)?)
            }
            ConfigReaderConfiguration::DataStore {
                project_id,
                credentials_file_path,
            } => Box::from(
                DatastoreTriggerConfigLoader::from_credentials(project_id, credentials_file_path)
                    .await?,
            ),
        };

        Ok(r)
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
    pub async fn into_instance(self) -> Result<Box<dyn TriggerQueueWriter + Send>> {
        let r: Box<dyn TriggerQueueWriter + Send> = match self {
            QueueWriterConfiguration::Directory { path } => {
                Box::from(DirectoryTriggerQueueWriter::new(path)?)
            }
            QueueWriterConfiguration::PubSub {
                project_id,
                credentials_file_path,
                topic,
            } => Box::from(
                PubsubTriggerQueueWriter::from_credentials(
                    project_id,
                    credentials_file_path,
                    topic,
                )
                .await?,
            ),
        };

        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    macro_rules! parse_ok {
        ($t: ident, $(($name:ident, $eq_to:expr),)*) => {
            $(
                #[tokio::test]
                async fn $name() {
                    const DATA_RAW: &str =
                        include_str!(concat!("test_data/", stringify!($name), ".json"));

                    let deserialized: $t = serde_json::from_str(DATA_RAW).unwrap();
                    assert_eq!(deserialized, $eq_to);

                    // We don't care about whether it failed.
                    match deserialized.into_instance().await {
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

    #[tokio::test]
    async fn trigger_system_config() {
        let host = PluginHost::default();

        let expected_cfg = TriggerSystemConfiguration {
            config_reader: ConfigReaderConfiguration::File {
                file: PathBuf::from("a.json"),
            },
            queue_writer: QueueWriterConfiguration::Directory {
                path: PathBuf::from("bong/"),
            },
        };

        const DATA_RAW: &str = include_str!("test_data/trigger_ok.json");

        let deserialized: TriggerSystemConfiguration = serde_json::from_str(DATA_RAW).unwrap();
        assert_eq!(deserialized, expected_cfg);

        match deserialized.into_instance(Arc::from(host)).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
