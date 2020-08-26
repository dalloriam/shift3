use std::fmt;

use gcloud::{datastore::DatastoreClient, AuthProvider};
use protocol::{Rule, RuleID};

use crate::interface::ActionConfigReader;

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

pub struct DatastoreActionConfigLoader {
    client: DatastoreClient,
}

impl DatastoreActionConfigLoader {
    pub fn new(project_id: String, authenticator: AuthProvider) -> Self {
        Self {
            client: DatastoreClient::new(project_id, authenticator),
        }
    }
}

impl ActionConfigReader for DatastoreActionConfigLoader {
    type Error = Error;

    fn get_rule(&self, id: RuleID) -> Result<Rule, Self::Error> {
        let result: Option<Rule> = self
            .client
            .get(id)
            .map_err(|ds| Error::DatastoreError(format!("{:?}", ds)))?;

        match result {
            None => Err(Error::DatastoreError(String::from(""))), // TODO: Return a proper error
            Some(r) => Ok(r),
        }
    }
}
