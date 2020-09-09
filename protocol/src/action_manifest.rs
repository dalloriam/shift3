use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionManifest {
    pub rule: RuleID,
    pub action_type: String,
    pub data: String,
}
