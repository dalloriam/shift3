use std::sync::mpsc;

use toolkit::thread::StoppableThread;

use crate::{CfgLoader, QueueWriter};

/// The trigger system manages the operation of the trigger service.
/// It manages its own threads and resources.
pub struct TriggerSystem {
    handle: StoppableThread<()>,
}

impl TriggerSystem {
    /// Creates a new trigger system.
    pub fn start<CfgErr, QueueErr>(
        cfg_loader: CfgLoader<CfgErr>,
        queue_writer: QueueWriter<QueueErr>,
    ) -> Self {
        Self {
            handle: StoppableThread::spawn(TriggerSystem::start_manager_thread),
        }
    }

    fn start_manager_thread(stop_rx: mpsc::Receiver<()>) {
        unimplemented!()
    }
}
