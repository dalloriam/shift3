use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gcloud::{datastore::DatastoreClient, AuthProvider};

use crate::interface::{TriggerConfigLoader, TriggerConfiguration};

#[derive(Debug)]
pub enum Error {
    DatastoreError(String),
    SerializationError(serde_json::Error),
    IOError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

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
    type Error = Error;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error> {
        let results = self
            .client
            .get_all()
            .map_err(|ds| Error::DatastoreError(format!("{:?}", ds)))?;
        Ok(results)
    }
}

/// Reads trigger configurations from a file.
pub struct FileTriggerConfigLoader {
    path: PathBuf,
}

impl TriggerConfigLoader for FileTriggerConfigLoader {
    type Error = Error;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error> {
        let handle = fs::File::open(&self.path).map_err(Error::IOError)?;
        let value: Vec<TriggerConfiguration> =
            serde_json::from_reader(handle).map_err(Error::SerializationError)?;

        Ok(value)
    }
}
