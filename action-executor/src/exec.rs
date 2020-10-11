use std::collections::HashMap;

use anyhow::Result;

use protocol::ActionManifest;

use crate::builtins;

pub trait ActionExecutor {
    fn execute(&self, manifest: ActionManifest) -> Result<()>;
}

pub type ExecutorObj = Box<dyn ActionExecutor>;

enum ActionType {
    SystemNotify,
    UserDefined(String),
}

impl From<String> for ActionType {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "system_notify" => ActionType::SystemNotify,
            _ => ActionType::UserDefined(s),
        }
    }
}

pub fn load_executors() -> Result<HashMap<String, ExecutorObj>> {
    // TODO: Use config here instead of hardcoding.
    let mut executors: HashMap<String, Box<dyn ActionExecutor>> = HashMap::new();

    executors.insert(
        String::from("notify"),
        Box::from(builtins::NotifyAction::default()),
    );

    Ok(executors)
}
