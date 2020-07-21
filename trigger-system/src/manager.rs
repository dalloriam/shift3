use crate::interface::{TriggerConfigLoader, TriggerQueueWriter};

/// The trigger manager is the "main" thread of the trigger system.
pub struct TriggerManager<T, Q>
where
    T: TriggerConfigLoader,
    Q: TriggerQueueWriter,
{
    cfg_loader: T,
    queue_writer: Q,
}

impl<T, Q> TriggerManager<T, Q>
where
    T: TriggerConfigLoader,
    Q: TriggerQueueWriter,
{
    pub fn new(cfg_loader: T, queue_writer: Q) -> Self {
        TriggerManager {
            cfg_loader,
            queue_writer,
        }
    }
}
