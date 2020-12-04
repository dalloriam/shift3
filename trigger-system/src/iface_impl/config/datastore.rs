use std::path::Path;

use anyhow::Result;

use async_std::sync::Mutex;

use async_trait::async_trait;

use gcloud::AuthProvider;

use google_cloud::datastore;

use crate::interface::{TriggerConfigLoader, TriggerConfiguration};

pub struct DatastoreTriggerConfigLoader {
    client: Mutex<datastore::Client>,
}

impl DatastoreTriggerConfigLoader {
    pub async fn new(project_id: String, authenticator: AuthProvider) -> Result<Self> {
        let client = datastore::Client::from_credentials(project_id, authenticator.into()).await?;
        Ok(Self {
            client: Mutex::from(client),
        })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
    ) -> Result<Self> {
        let auth = AuthProvider::from_json_file(credentials_file_path)?;
        Self::new(project_id, auth).await
    }
}

#[async_trait]
impl TriggerConfigLoader for DatastoreTriggerConfigLoader {
    async fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>> {
        let mut client_guard = self.client.lock().await;
        let configs: Result<Vec<TriggerConfiguration>> = client_guard
            .query(datastore::Query::new("TriggerConfiguration"))
            .await?
            .into_iter()
            .map(|e| {
                datastore::FromValue::from_value(e.into_properties()).map_err(anyhow::Error::new)
            })
            .collect();
        configs
    }
}
