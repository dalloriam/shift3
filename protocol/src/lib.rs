mod action_manifest;
pub mod rule;
pub mod trigger;

pub type RuleID = u64;

pub use action_manifest::ActionManifest;
pub use rule::Rule;
pub use trigger::{Trigger, TriggerConfiguration};
