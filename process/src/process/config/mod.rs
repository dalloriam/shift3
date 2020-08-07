mod trigger;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    systems: Vec<SystemConfiguration>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum SystemConfiguration {
    TriggerSystem(trigger::TriggerSystemConfiguration),
}
