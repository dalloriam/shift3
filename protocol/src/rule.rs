use serde::{Deserialize, Serialize};

use google_cloud::datastore::{FromValue, IntoValue};

use crate::RuleID;

#[derive(Clone, Debug, FromValue, IntoValue, Deserialize, Serialize)]
pub struct Rule {
    pub id: RuleID,
    pub trigger_config_id: i64,
    pub action_config: String,
    pub action_type: String,
}
