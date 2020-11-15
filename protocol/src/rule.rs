use serde::{Deserialize, Serialize};

use google_cloud::datastore::{FromValue, IntoValue};

#[derive(Clone, Debug, FromValue, IntoValue, Deserialize, Serialize)]
#[datastore(rename_all = "snake_case")]
pub struct Rule {
    pub trigger_config_id: i64,
    pub action_config: String,
    pub action_type: String,
}
