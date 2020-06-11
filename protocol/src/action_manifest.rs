use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionManifest {
    rule: RuleID,
    action_type: String,
    data: Vec<u8>,
}
