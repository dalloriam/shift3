use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{RuleID, Variant};

type ActionData = HashMap<String, Variant>;

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionManifest {
    rule: RuleID,
    action_type: String,
    data: ActionData,
}
