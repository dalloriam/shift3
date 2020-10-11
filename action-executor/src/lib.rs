mod builtins;
mod exec;
pub mod iface_impl;
mod interfaces;
mod manager;
mod system;

pub use interfaces::ActionManifestQueueReader;
pub use system::ExecutorSystem;

type BoxedQueueReader = Box<dyn ActionManifestQueueReader + Send>;
