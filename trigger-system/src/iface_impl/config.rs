use std::fmt;

use gcloud::{datastore::DatastoreClient, AuthProvider};

use crate::interface::{TriggerConfigLoader, TriggerConfiguration};

#[derive(Debug)]
pub enum Error {
    DatastoreError(String),
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
