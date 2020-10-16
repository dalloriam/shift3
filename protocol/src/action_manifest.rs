use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ActionManifest {
    pub rule: RuleID,
    pub action_type: String,
    pub data: String,
}
