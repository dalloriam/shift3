use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_std::sync::Mutex;

use async_trait::async_trait;

use anyhow::{anyhow, ensure, Error, Result};

use gcloud::AuthProvider;

use google_cloud::datastore;

use protocol::Rule;

use toolkit::db::sled::{EntityStore, SledStore};

use crate::interface::ActionConfigReader;

pub struct DatastoreActionConfigLoader {
    client: Mutex<datastore::Client>,
}

impl DatastoreActionConfigLoader {
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
        let authenticator = AuthProvider::from_json_file(credentials_file_path)?;
        Self::new(project_id, authenticator).await
    }
}

#[async_trait]
impl ActionConfigReader for DatastoreActionConfigLoader {
    async fn get_rule(&self, id: &str) -> Result<Rule> {
        let mut client_guard = self.client.lock().await;
        let client = &mut (*client_guard);

        // TODO: Not a fan of entering the kind as a string, it's error-prone...
        let key = datastore::Key::new("Rule").id(id.clone());
        let result: Option<Rule> = client.get(key).await?;

        match result {
            None => Err(Error::msg(format!("Rule with id '{}' not found.", id))),
            Some(r) => Ok(r),
        }
    }
}

/// Reads action configurations from a directory.
pub struct FileActionConfigReader {
    path: PathBuf,
}

impl FileActionConfigReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        ensure!(
            path.as_ref().exists(),
            format!("{:?} does not exist", path.as_ref())
        );

        Ok(Self {
            path: PathBuf::from(path.as_ref()),
        })
    }
}

#[async_trait]
impl ActionConfigReader for FileActionConfigReader {
    async fn get_rule(&self, id: &str) -> Result<Rule> {
        let path = self.path.join(format!("action_config_{}.txt", id));

        let data = fs::read_to_string(path)?;
        let rule = serde_json::from_str(data.as_ref())?;

        Ok(rule)
    }
}

pub struct EmbeddedActionConfigReader {
    rules: EntityStore<Rule>,
}

impl EmbeddedActionConfigReader {
    pub fn new(db: Arc<SledStore>) -> Result<EmbeddedActionConfigReader> {
        let rules: EntityStore<Rule> = db.entity("Rule")?;
        Ok(Self { rules })
    }
}

#[async_trait]
impl ActionConfigReader for EmbeddedActionConfigReader {
    async fn get_rule(&self, id: &str) -> Result<Rule> {
        let r = self.rules.get(id)?;

        match r {
            Some(rule) => Ok(rule),
            None => Err(anyhow!("Rule doesn't exist")),
        }
    }
}
