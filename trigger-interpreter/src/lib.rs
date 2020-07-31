//! Backing library for the Trigger Interpretation Service.

// Module declarations.
pub mod iface_impl;
mod interface;
mod templating;

// Public crate interface.
pub use interface::{ActionConfigReader, ActionManifestQueueWriter, TriggerQueueReader};
pub use templating::render_template;
