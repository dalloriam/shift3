use gcloud::datastore::DatastoreEntity;

use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(Clone, DatastoreEntity, Debug, Deserialize, PartialEq, Serialize)]
pub struct TriggerConfiguration {
    pub id: u64,
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
