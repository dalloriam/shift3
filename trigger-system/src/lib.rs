mod builtins;
mod exec;
pub mod iface_impl;
mod interface;
mod manager;
mod system;

// Public interface.
pub use interface::{TriggerConfigLoader, TriggerQueueWriter};
pub use system::{TriggerSystem, TriggerSystemConfig};

type BoxedCfgLoader = Box<dyn TriggerConfigLoader + Send>;
type BoxedQueueWriter = Box<dyn TriggerQueueWriter + Send>;

#[cfg(test)]
mod tests;
