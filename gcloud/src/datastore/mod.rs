//! Simple opinionated wrapper for interacting with google datastore.

mod client;
mod to_entity;
mod value;

// Custom-derive import & re-export.
extern crate datastore_to_entity_derive;
pub use datastore_to_entity_derive::ToEntity;

use to_entity::Error as EntityConversionError;

// Public package interface.
pub use client::DatastoreClient;
pub use to_entity::{DSEntity, ToEntity};
pub use value::DatastoreValue;
