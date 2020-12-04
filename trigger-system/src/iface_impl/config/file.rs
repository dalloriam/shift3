use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use anyhow::{ensure, Result};

use crate::interface::{TriggerConfigLoader, TriggerConfiguration};

/// Reads trigger configurations from a file.
pub struct FileTriggerConfigLoader {
    path: PathBuf,
}

impl FileTriggerConfigLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        ensure!(
            path.as_ref().exists(),
            format!("{:?} doesn't exist", path.as_ref())
        );

        Ok(FileTriggerConfigLoader {
            path: PathBuf::from(path.as_ref()),
        })
    }
}

#[async_trait]
impl TriggerConfigLoader for FileTriggerConfigLoader {
    async fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>> {
        let handle = fs::File::open(&self.path)?;
        let value: Vec<TriggerConfiguration> = serde_json::from_reader(handle)?;
        Ok(value)
    }
}
