use std::sync::mpsc;

use toolkit::thread::StoppableThread;

use crate::manager::TriggerManager;
use crate::{CfgLoader, QueueWriter};

/// The trigger system manages the operation of the trigger service.
/// It manages its own threads and resources.
pub struct TriggerSystem {
    handle: StoppableThread<()>,
}

impl TriggerSystem {
    /// Creates a new trigger system.
    pub fn start<CfgErr: Send, QueueErr: Send>(
        cfg_loader: CfgLoader<CfgErr>,
        queue_writer: QueueWriter<QueueErr>,
    ) -> Self {
        Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                TriggerSystem::start_manager_thread(stop_rx, cfg_loader, queue_writer)
            }),
        }
    }

    fn start_manager_thread<CfgErr: Send, QueueErr: Send>(
        stop_rx: mpsc::Receiver<()>,
        cfg_loader: CfgLoader<CfgErr>,
        queue_writer: QueueWriter<QueueErr>,
    ) {
        let manager = TriggerManager::new(cfg_loader, queue_writer);
    }
}
