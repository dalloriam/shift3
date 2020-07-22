use std::collections::HashSet;

use anyhow::Result;

use protocol::{Trigger, TriggerConfiguration};

pub struct DirectoryWatchTrigger {
    seen_files: HashSet<String>,
}

impl DirectoryWatchTrigger {
    pub fn new() -> Self {
        Self {
            seen_files: HashSet::new(),
        }
    }

    pub fn execute(&mut self, cfg: &TriggerConfiguration) -> Result<Vec<Trigger>> {
        let mut results = Vec::new();

        Ok(results)
    }
}
