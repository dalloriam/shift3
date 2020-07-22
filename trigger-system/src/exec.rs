use anyhow::Result;

use protocol::{Trigger, TriggerConfiguration};

enum TriggerType {
    DirectoryWatch,
    UserDefined(String), // Unimplemented right now - will be used for allowing users to load custom triggers with plugins.
}

impl From<String> for TriggerType {
    fn from(s: String) -> TriggerType {
        match s.as_ref() {
            "directory_watch" => TriggerType::DirectoryWatch,
            _ => TriggerType::UserDefined(s),
        }
    }
}

pub struct TriggerExecutor {}

impl TriggerExecutor {
    pub fn new() {}
}
