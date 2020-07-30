use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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
        let data = fs::read_to_string(&self.path)?;
        let configs: Vec<TriggerConfiguration> = serde_json::from_str(&data)?;
        Ok(configs)
    }
}

pub struct DiskQueueWriter {
    path: PathBuf,
    counter: AtomicU64,
}

impl DiskQueueWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> DiskQueueWriter {
        DiskQueueWriter {
            path: PathBuf::from(path.as_ref()),
            counter: AtomicU64::new(0),
        }
    }
}

impl TriggerQueueWriter for DiskQueueWriter {
    type Error = io::Error;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error> {
        let value = self.counter.fetch_add(1, Ordering::SeqCst);
        let path = self.path.join(format!("trigger_{}.txt", value));

        let file_handle = fs::File::create(path)?;
        serde_json::to_writer(file_handle, &trigger)?;
        Ok(())
    }
}
