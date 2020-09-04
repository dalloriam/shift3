use toolkit::thread::StoppableThread;

use crate::manager::ExecutorManager;
use crate::BoxedQueueReader;

pub struct ExecutorSystem {
    handle: StoppableThread<()>,
}

impl ExecutorSystem {
    pub fn start(queue_reader: BoxedQueueReader) -> Self {
        log::debug!("starting system");

        let sys = Self {
            handle: StoppableThread::spawn(move |stop_rx| {
                match ExecutorManager::new(stop_rx, queue_reader) {
                    Ok(mut e) => e.start(),
                    Err(err) => log::error!("failed to start the manager: {:?}", err),
                }
            }),
        };

        log::info!("system started");

        sys
    }
}
