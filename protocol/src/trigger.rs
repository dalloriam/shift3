use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Debug, Deserialize, Serialize)]
pub struct TriggerConfiguration {
    pub id: u64,
    pub rule: RuleID,
    pub trigger_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trigger {
    rule: RuleID,
    trigger_type: String,
    data: Vec<u8>,
}
