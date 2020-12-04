use google_cloud::datastore::{FromValue, IntoValue};

use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Clone, FromValue, IntoValue, Debug, Deserialize, PartialEq, Serialize)]
#[datastore(rename_all = "snake_case")]
pub struct TriggerConfiguration {
    pub id: i64,
    pub rule: RuleID,
    pub trigger_type: String,
    pub data: String, // JSON-encoded for now, willing to discuss formatting or alternatives later.
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Trigger {
    pub rule: RuleID,
    pub trigger_type: String,
    pub data: String, // Again, JSON-encoded for now.
}
