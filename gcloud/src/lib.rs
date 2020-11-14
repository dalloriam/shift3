#![warn(missing_docs)] // Force documentation of all public interfaces.
//! User-friendly GCloud wrappers.

pub mod auth;
pub mod pubsub;

pub use auth::{AuthError, AuthProvider};
//pub mod pub_sub;
