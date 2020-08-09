use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};

use gcloud::AuthProvider;

use trigger_interpreter::iface_impl::{
    DatastoreActionConfigLoader, PubSubActionManifestWriter, PubSubTriggerReader,
};
use trigger_interpreter::TriggerInterpreter;

const GCLOUD_KEY_FILE_ENV: &str = "GOOGLE_APPLICATION_CREDENTIALS";
const GCLOUD_PROJECT_ID_ENV: &str = "GOOGLE_PROJECT_ID";
const GOOGLE_PUBSUB_SUBSCRIPTION_ENV: &str = "GOOGLE_PUBSUB_SUBSCRIPTION_ID";
const GOOGLE_PUBSUB_TOPIC_ID_ENV: &str = "GOOGLE_PUBSUB_TOPIC_ID";

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

fn gcloud_credentials_path() -> Result<String> {
    let gcloud_credentials_path = env::var(GCLOUD_KEY_FILE_ENV).context(format!(
        "Missing key file. Please set '{}'.",
        GCLOUD_KEY_FILE_ENV
    ))?;

    Ok(gcloud_credentials_path)
}

fn project_id() -> Result<String> {
    let project_id = env::var(GCLOUD_PROJECT_ID_ENV).context(format!(
        "Missing project ID. Please set '{}'",
        GCLOUD_PROJECT_ID_ENV
    ))?;

    Ok(project_id)
}

fn subscription_id() -> Result<String> {
    let subscription_id = env::var(GOOGLE_PUBSUB_SUBSCRIPTION_ENV).context(format!(
        "Missing PubSub subscription ID. Please set '{}'",
        GOOGLE_PUBSUB_SUBSCRIPTION_ENV
    ))?;

    Ok(subscription_id)
}

fn topic_id() -> Result<String> {
    let topic_id = env::var(GOOGLE_PUBSUB_TOPIC_ID_ENV).context(format!(
        "Missing PubSub topic ID. Please set '{}'",
        GOOGLE_PUBSUB_TOPIC_ID_ENV
    ))?;

    Ok(topic_id)
}

fn get_action_config_loader(
    project_id: String,
    gcloud_credentials_path: String,
) -> Result<DatastoreActionConfigLoader> {
    let action_config_loader = DatastoreActionConfigLoader::new(
        project_id,
        AuthProvider::from_json_file(gcloud_credentials_path)?,
    );

    Ok(action_config_loader)
}

fn get_queue_reader(
    project_id: String,
    gcloud_credentials_path: String,
) -> Result<PubSubTriggerReader> {
    let queue_reader = PubSubTriggerReader::new(
        project_id,
        AuthProvider::from_json_file(gcloud_credentials_path)?,
        subscription_id()?,
    );

    Ok(queue_reader)
}

fn get_queue_writer(
    project_id: String,
    gcloud_credentials_path: String,
) -> Result<PubSubActionManifestWriter> {
    let queue_writer = PubSubActionManifestWriter::new(
        project_id,
        AuthProvider::from_json_file(gcloud_credentials_path)?,
        topic_id()?,
    );

    Ok(queue_writer)
}

fn run() -> Result<()> {
    let project_id = project_id()?;
    let gcloud_credentials_path = gcloud_credentials_path()?;

    // TODO: Create a trigger interpreter
    let interpreter = TriggerInterpreter::start(
        get_queue_reader(project_id.clone(), gcloud_credentials_path.clone())?,
        get_action_config_loader(project_id.clone(), gcloud_credentials_path.clone())?,
        get_queue_writer(project_id, gcloud_credentials_path)?,
    );

    wait_until_ctrlc()?;

    interpreter.stop()?;

    Ok(())
}

fn main() {
    init_logger();
    if let Err(e) = run() {
        log::error!("Fatal: {}", e);
    }
}
