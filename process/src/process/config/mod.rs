mod trigger;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::Service;

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    pub systems: Vec<SystemConfiguration>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SystemConfiguration {
    Trigger(trigger::TriggerSystemConfiguration),
}

impl SystemConfiguration {
    pub fn into_instance(self) -> Result<Service> {
        match self {
            SystemConfiguration::Trigger(cfg) => cfg.into_instance(),
        }
    }
}
