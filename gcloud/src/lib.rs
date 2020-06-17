#![deny(missing_docs)] // Force documentation of all public interfaces.
//! User-friendly GCloud wrappers.

mod auth;
pub mod datastore;
mod https;

pub use auth::{AuthError, AuthProvider};
