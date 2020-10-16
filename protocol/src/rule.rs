use serde::{Deserialize, Serialize};

use gcloud::datastore::DatastoreEntity;

use crate::RuleID;

#[derive(Clone, Debug, DatastoreEntity, Deserialize, Serialize)]
pub struct Rule {
    pub id: RuleID,
    pub trigger_config_id: u64,
    pub action_config: String,
    pub action_type: String,
}
