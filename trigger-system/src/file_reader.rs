use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{TriggerConfigLoader, TriggerConfiguration};

pub struct FileTriggerReader {
    path: PathBuf,
}

impl FileTriggerReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileTriggerReader {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl TriggerConfigLoader for FileTriggerReader {
    type Error = io::Error;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error> {
        let file = fs::File::open(&self.path)?;

        let config: Vec<TriggerConfiguration> = serde_json::from_reader(file)?;

        Ok(config)
    }
}