#![warn(missing_docs)] // Force documentation of all public interfaces.
//! User-friendly GCloud wrappers.

/// Authentication utilities.
pub mod auth;

/// Pubsub client.
pub mod pubsub;

pub use auth::{AuthError, AuthProvider};
//pub mod pub_sub;
