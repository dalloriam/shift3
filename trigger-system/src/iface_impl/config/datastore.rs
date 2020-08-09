use std::path::Path;

use anyhow::Result;

use gcloud::{datastore::DatastoreClient, AuthProvider};

use crate::interface::{TriggerConfigLoader, TriggerConfiguration};

pub struct DatastoreTriggerConfigLoader {
    client: DatastoreClient,
}

impl DatastoreTriggerConfigLoader {
    pub fn new(project_id: String, authenticator: AuthProvider) -> Self {
        Self {
            client: DatastoreClient::new(project_id, authenticator),
        }
    }

    pub fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
    ) -> Result<Self> {
        let auth = AuthProvider::from_json_file(credentials_file_path)?;
        Ok(Self::new(project_id, auth))
    }
}

impl TriggerConfigLoader for DatastoreTriggerConfigLoader {
    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>> {
        let configs = self.client.get_all()?;
        Ok(configs)
    }
}
