mod action;
mod error;
mod trigger;

pub use action::ActionPlugin;
pub use error::Error;
pub use trigger::TriggerPlugin;
pub const PLUGIN_INIT_SYMBOL: &str = "init_plugin";

#[derive(Default)]
pub struct Plugin {
    pub actions: Vec<Box<dyn ActionPlugin>>,
    pub triggers: Vec<Box<dyn TriggerPlugin>>,
}

impl Plugin {
    pub fn new(actions: Vec<Box<dyn ActionPlugin>>, triggers: Vec<Box<dyn TriggerPlugin>>) -> Self {
        Plugin { actions, triggers }
    }
}
