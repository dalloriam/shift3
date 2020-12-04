use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};

use async_trait::async_trait;

use gcloud::{auth, pubsub};

use glob::glob;

use protocol::Trigger;

use serde::de::DeserializeOwned;

use toolkit::message::{Error as MessageError, Message};

use crate::interface::TriggerQueueReader;

pub struct PubSubTriggerReader {
    subscription: pubsub::Subscription<Trigger, pubsub::formats::JSON>,
}

impl PubSubTriggerReader {
    pub async fn new(
        project_id: String,
        authenticator: auth::AuthProvider,
        subscription_id: String,
    ) -> Result<Self> {
        let client = pubsub::Client::new(&project_id, authenticator).await?;
        let subscription = client.subscription(&subscription_id).await?;
        Ok(PubSubTriggerReader { subscription })
    }

    pub async fn from_credentials<P: AsRef<Path>>(
        project_id: String,
        credentials_file_path: P,
        subscription: String,
    ) -> Result<Self> {
        let authenticator = auth::AuthProvider::from_json_file(credentials_file_path)?;
        Self::new(project_id, authenticator, subscription).await
    }
}

#[async_trait]
impl TriggerQueueReader for PubSubTriggerReader {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        let msg = self.subscription.pull().await?;
        Ok(msg)
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

#[async_trait]
impl TriggerQueueReader for FileTriggerQueueReader {
    async fn pull_trigger(&self) -> Result<Option<Box<dyn Message<Trigger> + Send>>> {
        let entries: Vec<PathBuf> = glob(&format!(
            "{}/trigger_*.txt",
            self.path.to_string_lossy().as_ref()
        ))?
        .filter_map(|x| x.ok())
        .collect();

        match entries.first() {
            Some(path) => Ok(Some(Box::from(FileQueueMessage { path: path.clone() }))),
            None => Ok(None),
        }
    }
}

struct FileQueueMessage {
    path: PathBuf,
}

#[async_trait]
impl<T> Message<T> for FileQueueMessage
where
    T: DeserializeOwned + 'static,
{
    async fn ack(&mut self) -> std::result::Result<(), MessageError> {
        fs::remove_file(&self.path).map_err(|e| MessageError::AckError {
            message: e.to_string(),
        })?;
        Ok(())
    }

    fn data(&self) -> std::result::Result<T, MessageError> {
        let file = fs::File::open(&self.path).map_err(|e| MessageError::DeserializeError {
            message: e.to_string(),
        })?;

        let deserialized =
            serde_json::from_reader(file).map_err(|e| MessageError::DeserializeError {
                message: e.to_string(),
            })?;

        Ok(deserialized)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use protocol::Trigger;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn file_queue() {
        // Setup a test directory.
        let temp_dir = tempdir().unwrap();
        let f = fs::File::create(temp_dir.path().join("trigger_1.txt")).unwrap();
        let expected_trigger = Trigger {
            rule: 3,
            trigger_type: String::from("something"),
            data: String::from("bing bong"),
        };
        serde_json::to_writer(f, &expected_trigger).unwrap();

        // Test the queue
        let q = FileTriggerQueueReader::new(temp_dir.path()).unwrap();
        let mut message = q.pull_trigger().await.unwrap().unwrap();
        let actual_trigger = message.data().unwrap();
        message.ack().await.unwrap();
        assert_eq!(actual_trigger, expected_trigger);

        assert!(q.pull_trigger().await.unwrap().is_none())
    }
}
