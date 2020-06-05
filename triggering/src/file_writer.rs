use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{Trigger, TriggerQueueWriter};

pub struct FileTriggerWriter {
    path: PathBuf,
}

impl FileTriggerWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileTriggerWriter {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl TriggerQueueWriter for FileTriggerWriter {
    type Error = io::Error;

    fn push_trigger(&self, trigger: Trigger) -> Result<(), Self::Error> {
        let mut f_handle = if self.path.exists() {
            fs::File::open(&self.path)?
        } else {
            fs::File::create(&self.path)?
        };

        serde_json::to_writer(&f_handle, &trigger).map_err(|json_err| {
            io::Error::new(io::ErrorKind::Other, format!("Serde error: {}", json_err))
        })?;

        f_handle.write_all("\n".as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::FileTriggerWriter;

    #[test]
    fn test_writer_init() {
        let w = FileTriggerWriter::new("queue.json");
        assert_eq!(w.path, PathBuf::from("queue.json"));
    }
}
