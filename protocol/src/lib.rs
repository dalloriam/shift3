mod action_manifest;
pub mod rule;
pub mod trigger;
pub mod variant;

pub type RuleID = u64;

pub use action_manifest::ActionManifest;
pub use rule::{ActionConfiguration, Rule};
pub use trigger::{Trigger, TriggerConfiguration};
pub use variant::Variant;
