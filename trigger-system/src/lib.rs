mod file_reader;
mod file_writer;
mod interface;
mod trigger;

pub type RuleID = u64;
pub use file_reader::FileTriggerReader;
pub use file_writer::FileTriggerWriter;
pub use interface::{TriggerConfigLoader, TriggerQueueWriter};
pub use trigger::{Trigger, TriggerConfiguration};
