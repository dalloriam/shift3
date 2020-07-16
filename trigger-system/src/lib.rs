mod interface;
mod manager;
mod system;

// Private interface.
type CfgLoader<E> = Box<dyn TriggerConfigLoader<Error = E>>;
type QueueWriter<E> = Box<dyn TriggerConfigLoader<Error = E>>;

// Public interface.
pub use interface::{TriggerConfigLoader, TriggerQueueWriter};
pub use system::TriggerSystem;
