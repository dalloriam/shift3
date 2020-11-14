//! Message interface wrapping attributes of queue messages.

use async_trait::async_trait;

use serde::de::DeserializeOwned;

use snafu::Snafu;

#[allow(missing_docs)]
#[derive(Debug, Snafu)]
pub enum Error {
    AckError { message: String },
    DeserializeError { message: String },
}

type Result<T> = std::result::Result<T, Error>;

/// A trait representing a message that can be deserialized.
#[async_trait]
pub trait Message<T>
where
    T: DeserializeOwned,
{
    /// Acknowledge the message.
    async fn ack(&mut self) -> Result<()>;

    /// Deserialize the data in the message.
    fn data(&self) -> Result<T>;
}
