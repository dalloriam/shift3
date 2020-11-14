use async_std::sync::Mutex;

use async_trait::async_trait;

use google_cloud::pubsub;

use serde::{de::DeserializeOwned, Serialize};

use snafu::{ensure, ResultExt, Snafu};

use crate::auth::AuthProvider;

#[derive(Debug, Snafu)]
pub enum Error {
    // Message errors.
    AckError {
        message: String,
    },
    MessageDeserializeError {
        message: String,
    },
    MessageSerializeError {
        source: serde_json::Error,
    },
    MessagePublishError {
        source: pubsub::Error,
    },

    // Client Errors
    FailedToInitializeClient {
        source: pubsub::Error,
    },
    FailedToGetTopic {
        source: pubsub::Error,
    },
    #[snafu(display("topic '{}' doesn't exist", topic))]
    TopicDoesntExist {
        topic: String,
    },
}

type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Message<T>
where
    T: DeserializeOwned,
{
    async fn ack(&mut self) -> Result<()>;

    fn data(&self) -> Result<T>;
}

pub struct Client {
    client: Mutex<pubsub::Client>,
}

impl Client {
    pub async fn new(project_id: &str, authenticator: AuthProvider) -> Result<Client> {
        let client = pubsub::Client::from_credentials(project_id, authenticator.into())
            .await
            .context(FailedToInitializeClient)?;
        Ok(Client {
            client: Mutex::from(client),
        })
    }

    pub async fn topic(&self, topic_id: &str) -> Result<Topic> {
        let mut client_guard = self.client.lock().await;
        let client_ref = &mut (*client_guard);
        Topic::new(topic_id, client_ref).await
    }
}

pub struct Topic {
    topic: Mutex<pubsub::Topic>,
}

impl Topic {
    async fn new(topic_id: &str, client: &mut pubsub::Client) -> Result<Self> {
        let topic = client.topic(topic_id).await.context(FailedToGetTopic)?;

        ensure!(
            topic.is_some(),
            TopicDoesntExist {
                topic: String::from(topic_id)
            }
        );

        Ok(Topic {
            topic: Mutex::from(topic.unwrap()),
        })
    }

    pub async fn publish<T: Serialize>(&self, body: T) -> Result<()> {
        let data = serde_json::to_vec(&body).context(MessageSerializeError)?;

        let mut topic_guard = self.topic.lock().await;
        (*topic_guard)
            .publish(data)
            .await
            .context(MessagePublishError)?;

        Ok(())
    }
}
