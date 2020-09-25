use std::path::Path;

use anyhow::{Error, Result};
use gcloud::{pub_sub::PubSubClient, AuthProvider};
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

impl Clone for Box<dyn TriggerQueueReader + Send> {
    fn clone(&self) -> Box<dyn TriggerQueueReader + Send> {
        self.box_clone()
    }
}

impl TriggerQueueReader for PubSubTriggerReader {
    fn box_clone(&self) -> Box<dyn TriggerQueueReader + Send> {
        Box::new(PubSubTriggerReader {
            client: PubSubClient::new(self.project_id.clone(), self.authenticator.clone()),
            subscription_id: self.subscription_id.clone(),
            project_id: self.project_id.clone(),
            authenticator: self.authenticator.clone(),
        })
    }

    fn pull_trigger(&self) -> Result<Vec<(String, Trigger)>> {
        let result = self
            .client
            .pull(self.subscription_id.as_str(), 10) // TODO: Us config instead of hardcoded value
            .map_err(|ds| Error::msg(format!("{:?}", ds)))?;

        Ok(result)
    }

    fn acknowlege(&self, ack_ids: Vec<String>) -> Result<()> {
        self.client
            .acknowledge(ack_ids, self.subscription_id.as_str())
            .map_err(|ds| Error::msg(format!("{:?}", ds)))?;

        Ok(())
    }
}
