//! Simple opinionated wrapper for interacting with google datastore.

mod client;
mod to_entity;
mod value;

use to_entity::Error as EntityConversionError;

// Custom-derive import & re-export.
extern crate datastore_to_entity_derive;
pub use datastore_to_entity_derive::DatastoreEntity;

// Public package interface.
pub use client::DatastoreClient;
pub use to_entity::{DSEntity, DatastoreEntity};
pub use value::DatastoreValue;
