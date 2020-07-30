mod action_manifest;
mod trigger;

pub type RuleID = u64;

pub use action_manifest::ActionManifest;
pub use trigger::{Trigger, TriggerConfiguration};
