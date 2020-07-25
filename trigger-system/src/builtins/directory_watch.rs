use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use protocol::Trigger;

use crate::exec::TriggerExecutor;

#[derive(Debug, Deserialize, Serialize)]
pub struct DirectoryWatchPayload {
    directory: PathBuf,
}

pub struct DirectoryWatchTrigger {
    seen_files: HashSet<String>,
}

impl DirectoryWatchTrigger {
    pub fn new() -> Self {
        Self {
            seen_files: HashSet::new(), // TODO: Add a way for triggers to persist state somewhere. This way, we don't re-notify on every boot.
        }
    }
}

impl TriggerExecutor for DirectoryWatchTrigger {
    // TODO: Finding a way to convert encoded_payload to the desired type from the TriggerManager would be awesome.
    fn execute(&mut self, encoded_payload: &str) -> Result<Vec<Trigger>> {
        let payload: DirectoryWatchPayload = serde_json::from_str(encoded_payload)?;

        let results = Vec::new();

        log::info!("Directory Watch running");

        Ok(results)
    }
}
