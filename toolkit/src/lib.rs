#![warn(missing_docs)] // Force documentation of all public interfaces.

//! Generic utility library.

pub mod message;
//pub mod queue;
mod stop;
pub mod thread;

pub use stop::Stop;
