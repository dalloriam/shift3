#![warn(missing_docs)] // Force documentation of all public interfaces.
//! User-friendly GCloud wrappers.

mod auth;
mod https;

pub use auth::{AuthError, AuthProvider};
pub mod datastore;
