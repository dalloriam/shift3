use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Result};

use clap::Clap;

use process::Node;

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

fn main_loop(config_file_path: &PathBuf) -> Result<()> {
    let cfg_format = polyglot::Format::try_from(
        config_file_path
            .extension()
            .ok_or_else(|| anyhow!("Missing extension"))?
            .to_string_lossy()
            .as_ref(),
    )?;

    let cfg_handle = fs::File::open(config_file_path)?;
    let node = Node::start(polyglot::from_reader(cfg_handle, cfg_format)?)?;

    wait_until_ctrlc()?;

    node.stop()
}

#[derive(Clap, Debug)]
#[clap(
    version = "0.1.0",
    author = "William Dussault",
    author = "Laurent Leclerc-Poulin"
)]
pub struct CLIMain {
    /// Path to the node configuration file.
    #[clap(short = "c", long = "cfg")]
    cfg: PathBuf,
}

impl CLIMain {
    pub fn run(self) -> Result<()> {
        main_loop(&self.cfg)
    }
}

fn main() {
    init_logger();

    let cli = CLIMain::parse();
    if let Err(e) = cli.run() {
        log::error!("Fatal: {}", e);
    }
}
