use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};

use gcloud::AuthProvider;

use trigger_interpreter::iface_impl::{
    DatastoreActionConfigLoader, PubSubActionManifestWriter, PubSubTriggerReader,
};

use protocol::rule::ActionConfiguration;
use protocol::Variant;

const GCLOUD_KEY_FILE_ENV: &str = "GOOGLE_APPLICATION_CREDENTIALS";
const GCLOUD_PROJECT_ID_ENV: &str = "GOOGLE_PROJECT_ID";

fn wait_until_ctrlc() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {}

    Ok(())
}

fn get_config_loader() -> Result<()> {
    let gcloud_credentials_path = env::var(GCLOUD_KEY_FILE_ENV).context(format!(
        "Missing key file. Please set '{}'.",
        GCLOUD_KEY_FILE_ENV
    ))?;

    let project_id = env::var(GCLOUD_PROJECT_ID_ENV).context(format!(
        "Missing project ID. Please set '{}'",
        GCLOUD_PROJECT_ID_ENV
    ))?;

    let authenticator = AuthProvider::from_json_file(gcloud_credentials_path)?;

    DatastoreActionConfigLoader::new(project_id, authenticator);

    PubSubActionManifestWriter::new(project_id, authenticator);

    PubSubTriggerReader::new(project_id, authenticator);

    // TODO: Return the configs and load pass them to the trigger interpreter
    Ok(())
}

fn main() {
    // TODO: Create a trigger interpreter
    //let interpreter = TriggerInterpreter::start(get_config_loader()?);

    wait_until_ctrlc()?;

    sys.stop()?;

    Ok(())
}
