use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Error, Result};
use gcloud::{pub_sub::Message, pub_sub::PubSubClient, AuthProvider};
use glob::glob;
use protocol::Trigger;

use crate::interface::TriggerQueueReader;

pub struct PubSubTriggerReader {
    client: PubSubClient,
    subscription_id: String,
    project_id: String,
    authenticator: AuthProvider,
}

impl PubSubTriggerReader {
    pub fn new(project_id: String, authenticator: AuthProvider, subscription_id: String) -> Self {
        PubSubTriggerReader {
            client: PubSubClient::new(project_id.clone(), authenticator.clone()),
            subscription_id,
            project_id,
            authenticator,
        }
    }

    pub fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        subscription: String,
    ) -> Result<Self> {
        let authenticator = AuthProvider::from_json_file(credentials_file_path)?;
        Ok(Self::new(project_id, authenticator, subscription))
    }
}

impl Clone for Box<dyn TriggerQueueReader> {
    fn clone(&self) -> Box<dyn TriggerQueueReader> {
        self.box_clone()
    }
}

impl TriggerQueueReader for PubSubTriggerReader {
    fn box_clone(&self) -> Box<dyn TriggerQueueReader> {
        Box::new(PubSubTriggerReader {
            client: PubSubClient::new(self.project_id.clone(), self.authenticator.clone()),
            subscription_id: self.subscription_id.clone(),
            project_id: self.project_id.clone(),
            authenticator: self.authenticator.clone(),
        })
    }

    fn pull_trigger(&self) -> Result<Vec<Message<Trigger>>> {
        let result = self
            .client
            .pull(self.subscription_id.as_str(), 10) // TODO: Use config instead of hardcoded value
            .map_err(|ds| Error::msg(format!("{:?}", ds)))?;

        Ok(result)
    }
}

/// Reads triggers from a directory.
pub struct FileTriggerQueueReader {
    path: PathBuf,
}

impl FileTriggerQueueReader {
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

impl TriggerQueueReader for FileTriggerQueueReader {
    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>> {
        let entries = glob(&format!(
            "{}/trigger_*.txt",
            self.path.to_string_lossy().as_ref()
        ))
        .expect("Failed to read glob pattern")
        .filter_map(|x| x.ok());

        let mut rules: Vec<(String, Trigger)> = Vec::new();
        for path in entries {
            let data = fs::read_to_string(path.clone())?;
            // Adds the unnecessary acknowledge id and the trigger data to the vector
            rules.push((String::from(""), serde_json::from_str(data.as_ref())?));
            // Delete the file once it was read
            fs::remove_file(path)?
        }

        Ok(rules)
    }

    fn box_clone(&self) -> Box<dyn TriggerQueueReader> {
        Box::new(FileTriggerQueueReader {
            path: self.path.clone(),
        })
    }
}
