use std::sync::mpsc;

use toolkit::thread::StoppableThread;

use crate::interface::{TriggerConfigLoader, TriggerQueueWriter};
use crate::manager::TriggerManager;

/// The trigger system manages the operation of the trigger service.
/// It manages its own threads and resources.
pub struct TriggerSystem {
    handle: StoppableThread<()>,
}

impl TriggerSystem {
    /// Creates a new trigger system.
    pub fn start<
        T: 'static + TriggerConfigLoader + Send,
        Q: 'static + TriggerQueueWriter + Send,
    >(
        cfg_loader: T,
        queue_writer: Q,
    ) -> Self {
        Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                TriggerSystem::start_manager_thread(stop_rx, cfg_loader, queue_writer)
            }),
        }
    }

    fn start_manager_thread<T: TriggerConfigLoader + Send, Q: TriggerQueueWriter + Send>(
        stop_rx: mpsc::Receiver<()>,
        cfg_loader: T,
        queue_writer: Q,
    ) {
        let manager = TriggerManager::new(cfg_loader, queue_writer);
    }
}
