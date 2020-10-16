pub mod iface_impl;
mod interfaces;
mod manager;
mod system;

pub use interfaces::ActionManifestQueueReader;
pub use system::{ExecutorSystem, ExecutorSystemConfig};

type BoxedQueueReader = Box<dyn ActionManifestQueueReader + Send>;

#[cfg(test)]
mod tests;
