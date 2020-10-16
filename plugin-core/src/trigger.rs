use protocol::{Trigger, TriggerConfiguration};

use crate::Error;

pub trait TriggerPlugin {
    fn get_type(&self) -> &str;
    fn pull_trigger(&self, cfg: TriggerConfiguration) -> Result<Vec<Trigger>, Error>;
}
