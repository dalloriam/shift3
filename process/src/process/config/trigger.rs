use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use trigger_system::TriggerConfigLoader;

#[derive(Deserialize, Serialize)]
pub struct TriggerSystemConfiguration {
    config_reader: ConfigReaderConfiguration,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")] // ConfigReaderConfiguration::Disk{blah: blah} => {type: Disk, blah: blah}
pub enum ConfigReaderConfiguration {
    DataStore {},
    Disk { file: PathBuf },
}
