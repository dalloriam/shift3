use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Debug, Deserialize, Serialize)]
pub struct TriggerConfiguration {
    id: u64,
    rule: RuleID,
    trigger_type: String,
    data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trigger {
    rule: RuleID,
    trigger_type: String,
    data: Vec<u8>,
}
