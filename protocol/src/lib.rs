mod action_manifest;
pub mod rule;
pub mod trigger;

pub type RuleID = i64;

pub use action_manifest::ActionManifest;
pub use rule::Rule;
pub use trigger::{Trigger, TriggerConfiguration};
