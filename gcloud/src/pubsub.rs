use std::{marker, time};

use async_std::sync::Mutex;

use async_trait::async_trait;

use google_cloud::pubsub;

use serde::{de::DeserializeOwned, Serialize};

use snafu::{ensure, ResultExt, Snafu};

use toolkit::message::{Error as MessageError, Message};

use crate::auth::AuthProvider;

#[allow(missing_docs)]
#[derive(Debug, Snafu)]
pub enum Error {
    // Message errors.
    Ack {
        source: pubsub::Error,
    },
    MessageDeserialize {
        source: serde_json::Error,
    },
    MessageSerialize {
        source: serde_json::Error,
    },
    MessagePublish {
        source: pubsub::Error,
    },

    // Client Errors
    InitializeClient {
        source: pubsub::Error,
    },
    GetSubscription {
        source: pubsub::Error,
    },
    GetTopic {
        source: pubsub::Error,
    },
    #[snafu(display("subscription '{}' doesn't exist", subscription))]
    SubscriptionDoesntExist {
        subscription: String,
    },
    #[snafu(display("topic '{}' doesn't exist", topic))]
    TopicDoesntExist {
        topic: String,
    },
}

type Result<T> = std::result::Result<T, Error>;

/// Small submodule regrouping pubsub message formats .
pub mod formats {
    use super::*;

    /// JSON pubsub message.
    pub struct JSON {
        message: pubsub::Message,
    }

    #[async_trait]
    impl<T> Message<T> for JSON
    where
        T: DeserializeOwned + 'static,
    {
        async fn ack(&mut self) -> std::result::Result<(), MessageError> {
            self.message
                .ack()
                .await
                .map_err(|e| MessageError::AckError {
                    message: e.to_string(),
                })?;
            Ok(())
        }

        fn data(&self) -> std::result::Result<T, MessageError> {
            let deserialized = serde_json::from_slice(self.message.data()).map_err(|e| {
                MessageError::DeserializeError {
                    message: e.to_string(),
                }
            })?;
            Ok(deserialized)
        }
    }

    impl From<pubsub::Message> for JSON {
        fn from(m: pubsub::Message) -> JSON {
            JSON { message: m }
        }
    }
}

/// A pubsub client.
pub struct Client {
    client: Mutex<pubsub::Client>,
}

impl Client {
    /// Initialize the pubsub client.
    pub async fn new(project_id: &str, authenticator: AuthProvider) -> Result<Client> {
        let client = pubsub::Client::from_credentials(project_id, authenticator.into())
            .await
            .context(InitializeClientSnafu)?;
        Ok(Client {
            client: Mutex::from(client),
        })
    }

    /// Get an existing subscription.
    pub async fn subscription<T, Format>(
        &self,
        subscription_id: &str,
    ) -> Result<Subscription<T, Format>>
    where
        T: DeserializeOwned + Send + 'static,
        Format: Message<T> + From<pubsub::Message> + Send + 'static,
    {
        let mut client_guard = self.client.lock().await;
        let client_ref = &mut (*client_guard);
        Subscription::new(subscription_id, client_ref).await
    }

    /// Get an existing topic.
    pub async fn topic(&self, topic_id: &str) -> Result<Topic> {
        let mut client_guard = self.client.lock().await;
        let client_ref = &mut (*client_guard);
        Topic::new(topic_id, client_ref).await
    }
}

/// A pubsub topic.
pub struct Topic {
    topic: Mutex<pubsub::Topic>,
}

impl Topic {
    async fn new(topic_id: &str, client: &mut pubsub::Client) -> Result<Self> {
        let topic = client.topic(topic_id).await.context(GetTopicSnafu)?;

        ensure!(
            topic.is_some(),
            TopicDoesntExistSnafu {
                topic: String::from(topic_id)
            }
        );

        Ok(Topic {
            topic: Mutex::from(topic.unwrap()),
        })
    }

    /// Publish a message to this topic.
    pub async fn publish<T: Serialize>(&self, body: T) -> Result<()> {
        let data = serde_json::to_vec(&body).context(MessageSerializeSnafu)?;

        let mut topic_guard = self.topic.lock().await;
        (*topic_guard)
            .publish(data)
            .await
            .context(MessagePublishSnafu)?;

        Ok(())
    }

    // TODO: Batch publish
}

/// A pubsub subscription.
pub struct Subscription<T, Format>
where
    T: DeserializeOwned + Send + 'static,
    Format: Message<T> + From<pubsub::Message> + Send + 'static,
{
    subscription: Mutex<pubsub::Subscription>,

    // Helpers to help the compiler see that a subscription can pull a single message type in a single format.
    phantom_type: marker::PhantomData<T>,
    phantom_format: marker::PhantomData<Format>,
}

impl<T, Format> Subscription<T, Format>
where
    T: DeserializeOwned + Send + 'static,
    Format: Message<T> + From<pubsub::Message> + Send + 'static,
{
    async fn new(
        subscription_id: &str,
        client: &mut pubsub::Client,
    ) -> Result<Subscription<T, Format>> {
        let subscription = client
            .subscription(subscription_id)
            .await
            .context(GetSubscriptionSnafu)?;

        ensure!(
            subscription.is_some(),
            SubscriptionDoesntExistSnafu {
                subscription: String::from(subscription_id)
            }
        );

        Ok(Subscription {
            subscription: Mutex::from(subscription.unwrap()),

            phantom_type: Default::default(),
            phantom_format: Default::default(),
        })
    }

    /// Pull a message from the subscription.
    pub async fn pull(&self) -> Result<Option<Box<dyn Message<T> + Send>>> {
        let mut subscription_guard = self.subscription.lock().await;
        match subscription_guard
            .receive_timeout(time::Duration::from_secs(5))
            .await
        {
            Some(m) => Ok(Some(Box::from(Format::from(m)))),
            None => Ok(None),
        }
    }
}
