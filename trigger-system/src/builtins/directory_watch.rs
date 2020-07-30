use std::collections::{HashMap, HashSet};
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
    seen_files: HashMap<u64, HashSet<PathBuf>>,
}

impl DirectoryWatchTrigger {
    pub fn new() -> Self {
        Self {
            seen_files: HashMap::new(), // TODO: Add a way for triggers to persist state somewhere. This way, we don't re-notify on every boot.
        }
    }
}

impl TriggerExecutor for DirectoryWatchTrigger {
    fn execute(&mut self, cfg: &TriggerConfiguration) -> Result<Vec<Trigger>> {
        let payload: DirectoryWatchPayload = serde_json::from_str(&cfg.data)?;

        match self.seen_files.get_mut(&cfg.rule) {
            Some(seen_files) => {
                let mut results = Vec::new();

                for entry in fs::read_dir(&payload.directory)?.filter_map(Result::ok) {
                    if !seen_files.contains(&entry.path()) {
                        // Add a trigger.
                        seen_files.insert(entry.path());
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
            None => {
                let mut initial_files = HashSet::new();

                for entry in fs::read_dir(&payload.directory)?.filter_map(Result::ok) {
                    initial_files.insert(entry.path());
                }

                self.seen_files.insert(cfg.rule, initial_files);

                Ok(Vec::new())
            }
        }
    }
}
