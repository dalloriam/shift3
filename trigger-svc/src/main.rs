mod disk;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;

use trigger_system::TriggerSystem;

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

fn run_system() -> Result<()> {
    let sys = TriggerSystem::start(
        disk::DiskConfigLoader::new("tmp/configs.json"),
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
