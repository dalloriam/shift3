mod builtins;
mod exec;
pub mod iface_impl;
mod interface;
mod manager;
mod system;

// Public interface.
pub use interface::{TriggerConfigLoader, TriggerQueueWriter};
pub use system::TriggerSystem;

#[cfg(test)]
mod tests;
