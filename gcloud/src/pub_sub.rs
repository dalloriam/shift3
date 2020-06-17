use google_pubsub1::{PublishResponse, Pubsub};
use serde::{de, Serialize};
use snafu::{ResultExt, Snafu};

use crate::{https, AuthProvider};

#[derive(Debug, Snafu)]
pub enum PubSubError {
    #[snafu(display("Failed to serialize the data structure : {}", source))]
    FailedToSerializeDataStruct { source: serde_json::Error },

    #[snafu(display("Failed to deserialize the data structure : {}", source))]
    FailedToDeserializeDataStruct { source: serde_json::Error },

    #[snafu(display("Failed to decode the data structure : {}", source))]
    FailedToDecodeDataStruc { source: base64::DecodeError },

    #[snafu(display("Failed publish the topic : {}", source))]
    FailedToPublishTopic { source: google_pubsub1::Error },

    #[snafu(display("Failed pull the subscription : {}", source))]
    FailedToPullSubscription { source: google_pubsub1::Error },

    #[snafu(display("Unexpected empty response"))]
    ErrorEmptyResponse,
}

type Result<T> = std::result::Result<T, PubSubError>;

pub struct PubSubClient {
    lib: Pubsub<hyper::Client, AuthProvider>,

    project_id: String,
}

impl PubSubClient {
    pub fn new(project_id: &str, auth_provider: AuthProvider) -> PubSubClient {
        let pub_sub = Pubsub::new(https::new_tls_client(), auth_provider);

        PubSubClient {
            lib: pub_sub,
            project_id: String::from(project_id),
        }
    }

    pub fn publish<T>(&self, data: T, topic: &str) -> Result<PublishResponse>
    where
        T: Serialize,
    {
        let json_body = serde_json::to_vec(&data).context(FailedToSerializeDataStruct)?;

        let message = google_pubsub1::PubsubMessage {
            data: Some(base64::encode(json_body)),
            ..Default::default()
        };

        let request = google_pubsub1::PublishRequest {
            messages: Some(vec![message]),
        };

        let (_res, pub_resp) = self
            .lib
            .projects()
            .topics_publish(
                request,
                &format!("projects/{}/topics/{}", self.project_id, topic),
            )
            .doit()
            .context(FailedToPublishTopic)?;

        Ok(pub_resp)
    }

    pub fn pull<T>(&self, subscription: &str) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        let request = google_pubsub1::PullRequest {
            return_immediately: Some(false),
            max_messages: Some(1),
        };

        let (_resp, pull_resp) = self
            .lib
            .projects()
            .subscriptions_pull(
                request,
                &format!(
                    "projects/{}/subscriptions/{}",
                    self.project_id, subscription
                ),
            )
            .doit()
            .context(FailedToPullSubscription)?;

        let received_messages = pull_resp
            .received_messages
            .ok_or_else(|| PubSubError::ErrorEmptyResponse)?;
        let received_message = received_messages
            .first()
            .ok_or_else(|| PubSubError::ErrorEmptyResponse)?;
        let message = received_message
            .clone()
            .message
            .ok_or_else(|| PubSubError::ErrorEmptyResponse)?;
        let data = message
            .data
            .ok_or_else(|| PubSubError::ErrorEmptyResponse)?;

        let decoded = base64::decode(&data).context(FailedToDecodeDataStruc)?;

        let resp: T = serde_json::from_slice(&decoded).context(FailedToDeserializeDataStruct)?;

        Ok(resp)
    }
}
