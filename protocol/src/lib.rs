#![recursion_limit = "10"]

mod action_manifest;
mod trigger;
mod variant;

pub type RuleID = u64;

pub use action_manifest::ActionManifest;
pub use trigger::{Trigger, TriggerConfiguration};
pub use variant::Variant;
