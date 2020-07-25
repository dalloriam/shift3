use gcloud::datastore::DatastoreEntity;

use serde::{Deserialize, Serialize};

use crate::RuleID;

#[derive(DatastoreEntity, Debug, Deserialize, Serialize)]
pub struct TriggerConfiguration {
    pub id: u64,
    pub rule: RuleID,
    pub trigger_type: String,
    pub data: String, // JSON-encoded for now, willing to discuss formatting or alternatives later.
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trigger {
    rule: RuleID,
    trigger_type: String,
    data: Vec<u8>,
}
