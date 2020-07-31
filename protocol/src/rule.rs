use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{RuleID, Variant};

pub type ActionConfiguration = HashMap<String, Variant>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    id: RuleID,
    trigger_config_id: u64,
    action_config: ActionConfiguration,
}
