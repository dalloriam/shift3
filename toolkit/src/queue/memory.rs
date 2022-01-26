use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;

use serde::{de::DeserializeOwned, Serialize};

use snafu::{ResultExt, Snafu};

use crate::message::{Error as MsgError, Message};

#[derive(Debug, Snafu)]
pub enum Error {
    MessageDeserialize { source: serde_json::Error },
    MessageSerialize { source: serde_json::Error },
    MutexPoisoning { message: String },
}

type Result<T> = std::result::Result<T, Error>;

struct MemoryMessage {
    data: Vec<u8>,
}

impl MemoryMessage {
    fn new<T: Serialize>(payload: T) -> Result<MemoryMessage> {
        let data = serde_json::to_vec(&payload).context(MessageSerializeSnafu)?;
        Ok(MemoryMessage { data })
    }

    fn decode<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.data).context(MessageDeserializeSnafu)
    }
}

#[async_trait]
impl<T> Message<T> for MemoryMessage
where
    T: DeserializeOwned + 'static,
{
    async fn ack(&mut self) -> std::result::Result<(), MsgError> {
        // TODO: Actually ack.
        Ok(())
    }

    fn data(&self) -> std::result::Result<T, MsgError> {
        self.decode().map_err(|e| MsgError::DeserializeError {
            message: e.to_string(),
        })
    }
}

/// An in-memory queue for sending messages between components of the generic process.
pub struct MemoryQueue {
    persist_path: Option<PathBuf>,

    queue: Mutex<VecDeque<MemoryMessage>>,
}

impl MemoryQueue {
    /// Create a basic in-memory queue.
    pub fn new() -> MemoryQueue {
        MemoryQueue {
            persist_path: None,
            queue: Mutex::from(VecDeque::new()),
        }
    }

    /// Creates a persistent memory queue with the specified path.
    pub fn with_persist_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.persist_path = Some(path.into());
        self
    }

    fn lock(&self) -> Result<MutexGuard<VecDeque<MemoryMessage>>> {
        self.queue.lock().map_err(|e| Error::MutexPoisoning {
            message: e.to_string(),
        })
    }

    /// Publishes a message to the memory queue.
    pub fn publish<T: Serialize>(&self, body: T) -> Result<()> {
        let message = MemoryMessage::new(body)?;

        let mut queue_guard = self.lock()?;
        let queue = &mut queue_guard;

        queue.push_back(message);

        Ok(())
    }

    /// Pulls a single message from the queue.
    pub fn pull<T: DeserializeOwned + Send + 'static>(
        &self,
    ) -> Result<Option<Box<dyn Message<T> + Send>>> {
        let mut queue_guard = self.lock()?;
        let queue = &mut queue_guard;

        match queue.pop_front() {
            Some(m) => Ok(Some(Box::from(m))),
            None => Ok(None),
        }
    }
}

impl Default for MemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Person {
        first_name: String,
        last_name: String,
        age: usize,
    }

    #[test]
    pub fn test_push_pull_single_message() {
        let queue = MemoryQueue::new();

        let p1 = Person {
            first_name: String::from("John"),
            last_name: String::from("Doe"),
            age: 18,
        };

        queue.publish(&p1).unwrap();

        let msg: Option<Box<dyn Message<Person> + Send>> = queue.pull().unwrap();
        let p2 = msg.unwrap().data().unwrap();

        assert_eq!(&p1, &p2);
    }
}
