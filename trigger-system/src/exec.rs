use std::collections::HashMap;

use anyhow::Result;

use protocol::{Trigger, TriggerConfiguration};

use crate::builtins;

pub trait TriggerExecutor {
    fn execute(&mut self, payload: &TriggerConfiguration) -> Result<Vec<Trigger>>;
}

pub type ExecutorObj = Box<dyn TriggerExecutor>;

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

pub fn load_executors() -> Result<HashMap<String, ExecutorObj>> {
    // TODO: Use config here instead of hardcoding.
    let mut executors: HashMap<String, Box<dyn TriggerExecutor>> = HashMap::new();

    executors.insert(
        String::from("directory_watch"),
        Box::from(builtins::DirectoryWatchTrigger::new()),
    );

    Ok(executors)
}
