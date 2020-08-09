use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{RuleID, Variant};

pub type ActionConfiguration = HashMap<String, Variant>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub id: RuleID,
    pub trigger_config_id: u64,
    pub action_config: ActionConfiguration,
    pub action_type: String,
}
