//! Backing library for the Trigger Interpretation Service.

// Module declarations.
pub mod iface_impl;
mod interface;
mod interpreter;
mod manager;
mod templating;

// Public crate interface.
pub use interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader};
pub use interpreter::TriggerInterpreter;
pub use templating::render_template;

type BoxedCfgReader = Box<dyn ActionConfigReader + Send>;
type BoxedQueueReader = Box<dyn TriggerQueueReader + Send>;
type BoxedQueueWriter = Box<dyn ActionManifestQueueWriter + Send>;

#[cfg(test)]
mod tests;
