#![deny(missing_docs)] // Force documentation of all public interfaces.
//! User-friendly GCloud wrappers.

mod auth;
mod https;
mod pub_sub;

pub use auth::{AuthError, AuthProvider};
