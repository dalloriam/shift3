//! Simple wrapper for interacting with google pubsub.
use google_pubsub1::Pubsub;

use serde::{de, Serialize};

use snafu::{ResultExt, Snafu};

use crate::{https, AuthProvider};

#[derive(Debug, Snafu)]
#[allow(missing_docs)] // Otherwise, cargo will ask to document each field of each error, which is a bit overkill.
pub enum PubSubError {
    #[snafu(display("Failed to serialize the data structure : {}", source))]
    FailedToSerializeDataStruct { source: serde_json::Error },

    #[snafu(display("Failed to deserialize the data structure : {}", source))]
    FailedToDeserializeDataStruct { source: serde_json::Error },

    #[snafu(display("Failed to decode the data structure : {}", source))]
    FailedToDecodeDataStruct { source: base64::DecodeError },

    #[snafu(display("Failed to publish the topic : {}", message))]
    FailedToPublishTopic { message: String },

    #[snafu(display("Failed to acknowledge the message : {}", message))]
    FailedToAcknowledgeMessage { message: String },

    #[snafu(display("Failed to pull the subscription : {}", message))]
    FailedToPullSubscription { message: String },

    #[snafu(display("PubSubClient - Unexpected empty response"))]
    EmptyResponse,
}

type Result<T> = std::result::Result<T, PubSubError>;

/// Google Cloud Pub/Sub client
pub struct PubSubClient {
    pubsub_connection: Pubsub<hyper::Client, AuthProvider>,
    auth_provider: AuthProvider,
    project_id: String,
}

impl Clone for PubSubClient {
    fn clone(&self) -> Self {
        PubSubClient {
            pubsub_connection: Pubsub::new(https::new_tls_client(), self.auth_provider.clone()),
            auth_provider: self.auth_provider.clone(),
            project_id: self.project_id.clone(),
        }
    }
}

impl PubSubClient {
    /// Creates a new client using a project identifier and an authentication provider.
    pub fn new(project_id: String, auth_provider: AuthProvider) -> PubSubClient {
        let pub_sub = Pubsub::new(https::new_tls_client(), auth_provider.clone());

        PubSubClient {
            auth_provider,
            pubsub_connection: pub_sub,
            project_id,
        }
    }

    /// Publish an entity to a Pub/Sub topic.
    ///
    /// The function allows to push a JSON serializable entity to a Pub/Sub topic.
    /// Therefore, the entity must implement serde's Serialize trait.
    pub fn publish<Entity>(&self, data: Entity, topic: &str) -> Result<()>
    where
        Entity: Serialize,
    {
        let json_body = serde_json::to_vec(&data).context(FailedToSerializeDataStruct)?;

        let message = google_pubsub1::PubsubMessage {
            data: Some(base64::encode(json_body)),
            ..Default::default()
        };

        let request = google_pubsub1::PublishRequest {
            messages: Some(vec![message]),
        };

        self.pubsub_connection
            .projects()
            .topics_publish(
                request,
                &format!("projects/{}/topics/{}", self.project_id, topic),
            )
            .doit()
            .map_err(|e| PubSubError::FailedToPublishTopic {
                message: e.to_string(),
            })?;

        Ok(())
    }

    /// Acknowledges the reception of a or multiple messages from a Pub/Sub subscription.
    pub fn acknowledge(&self, ack_ids: Vec<String>, subscription: &str) -> Result<()> {
        let acknowledge_request = google_pubsub1::AcknowledgeRequest {
            ack_ids: Some(ack_ids),
        };

        self.pubsub_connection
            .projects()
            .subscriptions_acknowledge(
                acknowledge_request,
                &format!(
                    "projects/{}/subscriptions/{}",
                    self.project_id, subscription
                ),
            )
            .doit()
            .map_err(|e| PubSubError::FailedToAcknowledgeMessage {
                message: e.to_string(),
            })?;

        Ok(())
    }

    /// Pulls N entities from a Pub/Sub subscription.
    ///
    /// The function allows to pull JSON deserializable entities from a Pub/Sub subscription.
    /// Therefore, the entity must implement serde's DeserializeOwned trait.
    /// Returns a vector of a tuple of the ID needed to acknowledge the message and the message.
    pub fn pull<Entity>(
        &self,
        subscription: &str,
        max_batch_size: i32,
    ) -> Result<Vec<(String, Entity)>>
    where
        Entity: de::DeserializeOwned,
    {
        let request = google_pubsub1::PullRequest {
            return_immediately: Some(false),
            max_messages: Some(max_batch_size),
        };

        let (_resp, pull_resp) = self
            .pubsub_connection
            .projects()
            .subscriptions_pull(
                request,
                &format!(
                    "projects/{}/subscriptions/{}",
                    self.project_id, subscription
                ),
            )
            .doit()
            .map_err(|e| PubSubError::FailedToPullSubscription {
                message: e.to_string(),
            })?;

        if let Some(received_messages) = pull_resp.received_messages {
            let mut entities: Vec<(String, Entity)> = Vec::new();

            for received_message in received_messages {
                let message = received_message
                    .message
                    .as_ref()
                    .ok_or(PubSubError::EmptyResponse)?;

                if let Some(id) = received_message.ack_id {
                    let data = message.data.as_ref().ok_or(PubSubError::EmptyResponse)?;

                    let decoded = base64::decode(&data).context(FailedToDecodeDataStruct)?;

                    let entity: Entity =
                        serde_json::from_slice(&decoded).context(FailedToDeserializeDataStruct)?;

                    entities.push((id, entity));
                }
            }

            Ok(entities)
        } else {
            Ok(Vec::new())
        }
    }
}
