use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Error, Result};
use gcloud::{datastore::DatastoreClient, AuthProvider};
use protocol::{Rule, RuleID};

use crate::interface::ActionConfigReader;

pub struct DatastoreActionConfigLoader {
    client: DatastoreClient,
}

impl DatastoreActionConfigLoader {
    pub fn new(project_id: String, authenticator: AuthProvider) -> Self {
        Self {
            client: DatastoreClient::new(project_id, authenticator),
        }
    }

    pub fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
    ) -> Result<Self> {
        let authenticator = AuthProvider::from_json_file(credentials_file_path)?;
        Ok(Self::new(project_id, authenticator))
    }
}

impl ActionConfigReader for DatastoreActionConfigLoader {
    fn get_rule(&self, id: RuleID) -> Result<Rule> {
        let result: Option<Rule> = self
            .client
            .get(id)
            .map_err(|ds| Error::msg(format!("{:?}", ds)))?;

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

impl ActionConfigReader for FileActionConfigReader {
    fn get_rule(&self, id: RuleID) -> Result<Rule> {
        let path = self.path.join(format!("action_config_{}.txt", id));

        let data = fs::read_to_string(path)?;
        let rule = serde_json::from_str(data.as_ref())?;

        Ok(rule)
    }
}
