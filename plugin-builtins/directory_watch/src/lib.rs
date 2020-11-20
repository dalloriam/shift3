use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use plugin_core::{Error, TriggerPlugin};

use protocol::{RuleID, Trigger, TriggerConfiguration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct DirectoryWatchPayload {
    directory: PathBuf,
}

#[derive(Debug, Serialize)]
struct TriggerData {
    file_name: String,
}

#[derive(Default)]
pub struct DirectoryWatcher {
    seen_files: Mutex<HashMap<RuleID, HashSet<PathBuf>>>,
}

impl TriggerPlugin for DirectoryWatcher {
    fn get_type(&self) -> &str {
        "directory_watch"
    }

    fn pull_trigger(&self, cfg: &TriggerConfiguration) -> Result<Vec<Trigger>, Error> {
        let payload: DirectoryWatchPayload =
            serde_json::from_str(&cfg.data).map_err(|e| Error {
                message: e.to_string(),
            })?;

        let mut seen_files_guard = self.seen_files.lock().map_err(|e| Error {
            message: e.to_string(),
        })?;
        let seen_files = &mut (*seen_files_guard);

        match seen_files.get_mut(&cfg.rule) {
            Some(seen_files) => {
                let mut results = Vec::new();

                for entry in fs::read_dir(&payload.directory)
                    .map_err(|e| Error {
                        message: e.to_string(),
                    })?
                    .filter_map(Result::ok)
                {
                    if !seen_files.contains(&entry.path()) {
                        // Add a trigger.
                        seen_files.insert(entry.path());
                        results.push(Trigger {
                            rule: cfg.rule.clone(),
                            trigger_type: cfg.trigger_type.clone(),
                            data: serde_json::to_string(&TriggerData {
                                file_name: entry.file_name().to_string_lossy().to_string(),
                            })
                            .map_err(|e| Error {
                                message: e.to_string(),
                            })?,
                        })
                    }
                }
                Ok(results)
            }
            None => {
                let mut initial_files = HashSet::new();

                for entry in fs::read_dir(&payload.directory)
                    .map_err(|e| Error {
                        message: e.to_string(),
                    })?
                    .filter_map(Result::ok)
                {
                    initial_files.insert(entry.path());
                }

                seen_files.insert(cfg.rule.clone(), initial_files);

                Ok(Vec::new())
            }
        }
    }
}

plugin_core::export!((), (DirectoryWatcher));
