use crate::{CfgLoader, QueueWriter};

/// The trigger manager is the "main" thread of the trigger system.
pub struct TriggerManager<CfgErr, QueueErr> {
    cfg_loader: CfgLoader<CfgErr>,
    queue_writer: QueueWriter<QueueErr>,
}

impl<CfgErr, QueueErr> TriggerManager<CfgErr, QueueErr> {
    pub fn new(cfg_loader: CfgLoader<CfgErr>, queue_writer: QueueWriter<QueueErr>) -> Self {
        TriggerManager {
            cfg_loader,
            queue_writer,
        }
    }
}
