mod disk;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};

use gcloud::AuthProvider;

use trigger_system::iface_impl::DatastoreTriggerConfigLoader;
use trigger_system::TriggerSystem;

const GCLOUD_KEY_FILE_ENV: &str = "GOOGLE_APPLICATION_CREDENTIALS";
const GCLOUD_PROJECT_ID_ENV: &str = "GOOGLE_PROJECT_ID";

fn init_logger() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
}

fn wait_until_ctrlc() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {}

    Ok(())
}

fn get_config_loader() -> Result<DatastoreTriggerConfigLoader> {
    let gcloud_credentials_path = env::var(GCLOUD_KEY_FILE_ENV).context(format!(
        "Missing key file. Please set '{}'.",
        GCLOUD_KEY_FILE_ENV
    ))?;

    let gcloud_project_id = env::var(GCLOUD_PROJECT_ID_ENV).context(format!(
        "Missing project ID. Please set '{}'",
        GCLOUD_PROJECT_ID_ENV
    ))?;

    Ok(DatastoreTriggerConfigLoader::new(
        gcloud_project_id,
        AuthProvider::from_json_file(gcloud_credentials_path)?,
    ))
}

fn run_system() -> Result<()> {
    let sys = TriggerSystem::start(
        get_config_loader()?,
        disk::DiskQueueWriter::new("tmp/queue"),
    );

    wait_until_ctrlc()?;

    sys.stop()?;

    Ok(())
}

fn main() {
    init_logger();
    if let Err(e) = run_system() {
        log::error!("Fatal: {}", e);
    }
}
