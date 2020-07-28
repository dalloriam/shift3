use std::io;
use std::path::{Path, PathBuf};

use protocol::{Trigger, TriggerConfiguration};

use trigger_system::{TriggerConfigLoader, TriggerQueueWriter};

pub struct DiskConfigLoader {
    path: PathBuf,
}

impl DiskConfigLoader {
    pub fn new<P: AsRef<Path>>(path: P) -> DiskConfigLoader {
        DiskConfigLoader {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl TriggerConfigLoader for DiskConfigLoader {
    type Error = io::Error;

    fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>, Self::Error> {
        log::debug!("got all configs");
        // TODO: Implement.
        Ok(Vec::new())
    }
}

pub struct DiskQueueWriter {
    path: PathBuf,
}

impl DiskQueueWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> DiskQueueWriter {
        DiskQueueWriter {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl TriggerQueueWriter for DiskQueueWriter {
    type Error = io::Error;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error> {
        log::debug!("push: {:?}", trigger);
        Ok(())
    }
}
