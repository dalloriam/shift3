use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Error;

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
}

impl TriggerConfigLoader for DatastoreTriggerConfigLoader {
    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error> {
        let configs = self.client.get_all()?;
        Ok(configs)
    }
}

/// Reads trigger configurations from a file.
pub struct FileTriggerConfigLoader {
    path: PathBuf,
}

impl FileTriggerConfigLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileTriggerConfigLoader {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl TriggerConfigLoader for FileTriggerConfigLoader {
    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Error> {
        let handle = fs::File::open(&self.path)?;
        let value: Vec<TriggerConfiguration> = serde_json::from_reader(handle)?;
        Ok(value)
    }
}
