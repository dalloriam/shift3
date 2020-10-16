use std::io;
use std::process::Command;

use plugin_core::{ActionPlugin, Error, Plugin};

use protocol::ActionManifest;

use serde::Deserialize;

#[derive(Deserialize)]
struct NotifyPayload {
    title: String,
    body: String,
}

#[derive(Clone, Debug, Default)]
pub struct NotifyPlugin {}

impl ActionPlugin for NotifyPlugin {
    fn get_type(&self) -> &str {
        "notify"
    }

    fn execute_action(&self, manifest: ActionManifest) -> Result<(), Error> {
        let payload: NotifyPayload = serde_json::from_str(&manifest.data).map_err(|e| Error {
            message: e.to_string(),
        })?;

        let mut child_process = Command::new("notify-send")
            .arg(payload.title)
            .arg(payload.body)
            .spawn()
            .map_err(|e| Error {
                message: e.to_string(),
            })?;

        let exit_status = child_process.wait().map_err(|e| Error {
            message: e.to_string(),
        })?;

        if !exit_status.success() {
            Err(Error {
                message: String::from("non-zero status code"),
            })
        } else {
            Ok(())
        }
    }
}

#[no_mangle]
pub extern "C" fn init_plugin() -> Box<Plugin> {
    Box::new(Plugin::new(
        vec![Box::new(NotifyPlugin::default())],
        Vec::new(),
    ))
}
