pub mod iface_impl;
mod interfaces;
mod manager;
mod system;

pub use interfaces::ActionManifestQueueReader;

type BoxedQueueReader = Box<dyn ActionManifestQueueReader + Send>;
