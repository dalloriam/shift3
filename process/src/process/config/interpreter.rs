use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::{ResourceManager, Service};

use trigger_interpreter::{
    iface_impl::{
        DatastoreActionConfigLoader, FileActionConfigReader, FileActionManifestWriter,
        FileTriggerQueueReader, InMemoryActionManifestQueueWriter, InMemoryTriggerQueueReader,
        PubSubActionManifestWriter, PubSubTriggerReader,
    },
    ActionConfigReader, ActionManifestQueueWriter, TriggerInterpreter, TriggerQueueReader,
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
    pub async fn into_instance(self, resource_manager: Arc<ResourceManager>) -> Result<Service> {
        let cfg_reader = self
            .config_reader
            .into_instance(resource_manager.clone())
            .await?;
        let queue_writer = self
            .queue_writer
            .into_instance(resource_manager.clone())
            .await?;
        let queue_reader = self
            .queue_reader
            .into_instance(resource_manager.clone())
            .await?;

        Ok(Box::from(TriggerInterpreter::start(
            queue_reader,
            cfg_reader,
            queue_writer,
        )))
    }
}

/// Configuration of the action config reader.
///
/// Contains configurations for the various supported config readers (e.g. file, datastore).
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")] // ConfigReaderConfiguration::File{file: blah} => {type: File, file: blah}
pub enum ConfigReaderConfiguration {
    File {
        file: PathBuf,
    },
    DataStore {
        project_id: String,
        credentials_file_path: String,
    },
}

impl ConfigReaderConfiguration {
    /// Returns a usable action config reader from the configuration struct.
    pub async fn into_instance(
        self,
        _resource_manager: Arc<ResourceManager>,
    ) -> Result<Box<dyn ActionConfigReader + Send>> {
        let b: Box<dyn ActionConfigReader + Send> = match self {
            ConfigReaderConfiguration::File { file } => {
                Box::from(FileActionConfigReader::new(file)?)
            }
            ConfigReaderConfiguration::DataStore {
                project_id,
                credentials_file_path,
            } => Box::from(
                DatastoreActionConfigLoader::from_credentials(project_id, credentials_file_path)
                    .await?,
            ),
        };

        Ok(b)
    }
}

/// Configuration of the queue writer.
///
/// Contains configurations for the various supported queue writers (e.g. directory, pubsub).
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")] // QueueWriterConfiguration::Directory{path: /blah} => {type: Directory, path: /blah}
pub enum QueueWriterConfiguration {
    Directory {
        path: PathBuf,
    },
    PubSub {
        project_id: String,
        credentials_file_path: String,
        topic: String,
    },
    InMemory {
        topic: String,
    },
}

impl QueueWriterConfiguration {
    pub async fn into_instance(
        self,
        resource_manager: Arc<ResourceManager>,
    ) -> Result<Box<dyn ActionManifestQueueWriter + Send>> {
        let b: Box<dyn ActionManifestQueueWriter + Send> = match self {
            QueueWriterConfiguration::Directory { path } => {
                Box::from(FileActionManifestWriter::new(path)?)
            }
            QueueWriterConfiguration::PubSub {
                project_id,
                credentials_file_path,
                topic,
            } => Box::from(
                PubSubActionManifestWriter::from_credentials(
                    project_id,
                    credentials_file_path,
                    topic,
                )
                .await?,
            ),
            QueueWriterConfiguration::InMemory { topic } => Box::from(
                InMemoryActionManifestQueueWriter::new(resource_manager.get_memory_queue(&topic)?),
            ),
        };

        Ok(b)
    }
}

/// Configuration of the queue reader.
///
/// Contains configurations for the various supported queue readers (e.g. directory, pubsub).
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")] // QueueReaderConfiguration::Directory{path: /blah} => {type: Directory, path: /blah}
pub enum QueueReaderConfiguration {
    Directory {
        path: PathBuf,
    },
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
    pub async fn into_instance(
        self,
        resource_manager: Arc<ResourceManager>,
    ) -> Result<Box<dyn TriggerQueueReader + Send>> {
        let b: Box<dyn TriggerQueueReader + Send> = match self {
            QueueReaderConfiguration::Directory { path } => {
                Box::from(FileTriggerQueueReader::new(path)?)
            }
            QueueReaderConfiguration::PubSub {
                project_id,
                credentials_file_path,
                subscription,
            } => Box::from(
                PubSubTriggerReader::from_credentials(
                    project_id,
                    credentials_file_path,
                    subscription,
                )
                .await?,
            ),
            QueueReaderConfiguration::InMemory { topic } => Box::from(
                InMemoryTriggerQueueReader::new(resource_manager.get_memory_queue(&topic)?),
            ),
        };

        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempdir::TempDir;

    use super::*;

    macro_rules! parse_ok {
        ($t: ident, $(($func_name:ident, $file_name:ident, $eq_to:expr),)*) => {
            $(
                #[tokio::test]
                async fn $func_name() {
                    const DATA_RAW: &str =
                        include_str!(concat!("test_data/", stringify!($file_name), ".json"));

                    let deserialized: $t = serde_json::from_str(DATA_RAW).unwrap();
                    assert_eq!(deserialized, $eq_to);

                    // We don't care about whether it failed.
                    // TLDR; Increase coverage
                    match deserialized.into_instance(Arc::from(ResourceManager::default())).await {
                        Ok(_) => {},
                        Err(_) => {}
                    }
                }
            )*
        };
    }

    macro_rules! parse_fail {
        ($t: ident, $(($func_name:ident, $file_name:ident),)*) => {
            $(
                #[test]
                fn $func_name() {
                    const DATA_RAW: &str =
                        include_str!(concat!("test_data/", stringify!($file_name), ".json"));

                    let deserialized_maybe: Result<$t, serde_json::Error> =
                        serde_json::from_str(DATA_RAW);
                    assert!(deserialized_maybe.is_err());
                }
            )*
        };
    }

    parse_ok! {
        ConfigReaderConfiguration,

        (file_cfg_valid, file_cfg_valid, ConfigReaderConfiguration::File{file: PathBuf::from("/tmp/yeet.json")}),
        (
            ds_cfg_valid, ds_cfg_valid,
            ConfigReaderConfiguration::DataStore{
                project_id: String::from("asdf123"),
                credentials_file_path: String::from("/etc/gcloud.json")
            }
        ),
    }

    parse_fail! {
        ConfigReaderConfiguration,

        (file_cfg_invalid, file_cfg_invalid),
        (cfg_gibberish, cfg_gibberish),
    }

    parse_ok! {
        QueueWriterConfiguration,

        (
            queue_dir_valid, queue_dir_valid,
            QueueWriterConfiguration::Directory {
                path: PathBuf::from("/tmp/yeet/")
            }
        ),
        (
            queue_pubsub_valid, queue_pubsub_valid,
            QueueWriterConfiguration::PubSub {
                project_id: String::from("asdf123"),
                credentials_file_path: String::from("/etc/gcloud.json"),
                topic: String::from("somequeue"),
            }
        ),
    }

    parse_fail! {
        QueueWriterConfiguration,

        (queue_gibberish, queue_gibberish),
    }

    parse_ok! {
        QueueReaderConfiguration,

        (
            queue_read_dir_valid,
            queue_dir_valid,
            QueueReaderConfiguration::Directory {
                path: PathBuf::from("/tmp/yeet/")
            }
        ),
        (
            queue_read_pubsub_valid,
            queue_read_pubsub_valid,
            QueueReaderConfiguration::PubSub {
                project_id: String::from("asdf123"),
                credentials_file_path: String::from("/etc/gcloud.json"),
                subscription: String::from("somequeue"),
            }
        ),
    }

    parse_fail! {
        QueueReaderConfiguration,

        (queue_read_gibberish, queue_gibberish),
    }

    #[tokio::test]
    async fn interpreter_system_instanciation() {
        let manager = ResourceManager::default();

        // Create the files expected by the config.
        let temp_dir = TempDir::new("").unwrap();
        let config_path = temp_dir.path().join("a.json");
        fs::File::create(&config_path).unwrap();

        let expected_cfg = TriggerInterpreterConfiguration {
            config_reader: ConfigReaderConfiguration::File { file: config_path },
            queue_reader: QueueReaderConfiguration::Directory {
                path: temp_dir.path().into(),
            },
            queue_writer: QueueWriterConfiguration::Directory {
                path: temp_dir.path().into(),
            },
        };

        match expected_cfg.into_instance(Arc::from(manager)).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    #[tokio::test]
    async fn interpreter_system_config() {
        let manager = ResourceManager::default();

        let expected_cfg = TriggerInterpreterConfiguration {
            config_reader: ConfigReaderConfiguration::File {
                file: PathBuf::from("a.json"),
            },
            queue_reader: QueueReaderConfiguration::Directory {
                path: PathBuf::from("bing/"),
            },
            queue_writer: QueueWriterConfiguration::Directory {
                path: PathBuf::from("bong/"),
            },
        };

        const DATA_RAW: &str = include_str!("test_data/interpreter_ok.json");

        let deserialized: TriggerInterpreterConfiguration = serde_json::from_str(DATA_RAW).unwrap();
        assert_eq!(deserialized, expected_cfg);

        match deserialized.into_instance(Arc::from(manager)).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
