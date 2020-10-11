use std::process::Command;

use anyhow::{ensure, Result};

use serde::{Deserialize, Serialize};

use protocol::ActionManifest;

use crate::exec::ActionExecutor;

#[derive(Deserialize, Serialize)]
struct NotifyData {
    body: String,
    title: String,
}

#[derive(Default)]
pub struct NotifyAction {}

impl NotifyAction {
    fn notify(&self, title: &str, body: &str) -> Result<()> {
        let mut child_process = Command::new("notify-send").arg(title).arg(body).spawn()?;
        let exit_status = child_process.wait()?;

        ensure!(
            exit_status.success(),
            "notify-send returned a non-zero status code"
        );
        Ok(())
    }
}

impl ActionExecutor for NotifyAction {
    fn execute(&self, manifest: ActionManifest) -> Result<()> {
        let data: NotifyData = serde_json::from_str(&manifest.data)?;
        self.notify(&data.title, &data.body)
    }
}
