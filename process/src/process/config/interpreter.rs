use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use plugin_host::PluginHost;

use serde::{Deserialize, Serialize};

use crate::Service;

use trigger_interpreter::{
    iface_impl::{
        DatastoreActionConfigLoader, FileActionConfigReader, FileActionManifestWriter,
        FileTriggerQueueReader, PubSubActionManifestWriter, PubSubTriggerReader,
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
    pub fn into_instance(self, _plugin_host: Arc<PluginHost>) -> Result<Service> {
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
    pub fn into_instance(self) -> Result<Box<dyn ActionConfigReader + Send>> {
        match self {
            ConfigReaderConfiguration::File { file } => {
                Ok(Box::from(FileActionConfigReader::new(file)?))
            }
            ConfigReaderConfiguration::DataStore {
                project_id,
                credentials_file_path,
            } => Ok(Box::from(async_std::task::block_on(
                DatastoreActionConfigLoader::from_credentials(project_id, credentials_file_path),
            )?)),
        }
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
}

impl QueueWriterConfiguration {
    pub fn into_instance(self) -> Result<Box<dyn ActionManifestQueueWriter + Send>> {
        match self {
            QueueWriterConfiguration::Directory { path } => {
                Ok(Box::from(FileActionManifestWriter::new(path)?))
            }
            QueueWriterConfiguration::PubSub {
                project_id,
                credentials_file_path,
                topic,
            } => Ok(Box::from(async_std::task::block_on(
                PubSubActionManifestWriter::from_credentials(
                    project_id,
                    credentials_file_path,
                    topic,
                ),
            )?)),
        }
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
}

impl QueueReaderConfiguration {
    pub fn into_instance(self) -> Result<Box<dyn TriggerQueueReader + Send>> {
        match self {
            QueueReaderConfiguration::Directory { path } => {
                Ok(Box::from(FileTriggerQueueReader::new(path)?))
            }
            QueueReaderConfiguration::PubSub {
                project_id,
                credentials_file_path,
                subscription,
            } => Ok(Box::from(async_std::task::block_on(
                PubSubTriggerReader::from_credentials(
                    project_id,
                    credentials_file_path,
                    subscription,
                ),
            )?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ConfigReaderConfiguration, QueueReaderConfiguration, QueueWriterConfiguration};

    macro_rules! parse_ok {
        ($t: ident, $(($func_name:ident, $file_name:ident, $eq_to:expr),)*) => {
            $(
                #[test]
                fn $func_name() {
                    const DATA_RAW: &str =
                        include_str!(concat!("test_data/", stringify!($file_name), ".json"));

                    let deserialized: $t = serde_json::from_str(DATA_RAW).unwrap();
                    assert_eq!(deserialized, $eq_to);

                    // We don't care about whether it failed.
                    // TLDR; Increase coverage
                    match deserialized.into_instance() {
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
}
