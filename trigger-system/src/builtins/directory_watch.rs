use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use protocol::{Trigger, TriggerConfiguration};

use crate::exec::TriggerExecutor;

#[derive(Debug, Deserialize, Serialize)]
struct DirectoryWatchPayload {
    directory: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct TriggerData {
    file_name: String,
}

pub struct DirectoryWatchTrigger {
    seen_files: HashSet<PathBuf>,
}

impl DirectoryWatchTrigger {
    pub fn new() -> Self {
        Self {
            seen_files: HashSet::new(), // TODO: Add a way for triggers to persist state somewhere. This way, we don't re-notify on every boot.
        }
    }
}

impl TriggerExecutor for DirectoryWatchTrigger {
    fn execute(&mut self, cfg: &TriggerConfiguration) -> Result<Vec<Trigger>> {
        let payload: DirectoryWatchPayload = serde_json::from_str(&cfg.data)?;

        let mut results = Vec::new();

        for entry in fs::read_dir(&payload.directory)?.filter_map(Result::ok) {
            if !self.seen_files.contains(&entry.path()) {
                // Add a trigger.
                self.seen_files.insert(entry.path());
                results.push(Trigger {
                    rule: cfg.rule,
                    trigger_type: cfg.trigger_type.clone(),
                    data: serde_json::to_string(&TriggerData {
                        file_name: entry.file_name().to_string_lossy().to_string(),
                    })?,
                })
            }
        }
        Ok(results)
    }
}
