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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use serde_json::json;

    use tempdir::TempDir;

    fn run_test(config_body: &str, config_extension: &str) {
        let test_dir = TempDir::new("").unwrap();

        let cfg_path = test_dir
            .path()
            .join("config")
            .with_extension(config_extension);

        // Create the config file for the process.
        {
            let mut cfg_handle = fs::File::create(&cfg_path).unwrap();
            write!(cfg_handle, "{}", config_body).unwrap();
        }

        // Create the queue directory.
        fs::create_dir(test_dir.path().join("queue")).unwrap();

        // Create the trigger config json file.
        let trigger_config = json!({
            "id": 1,
            "rule": 42,
            "trigger_type": "directory_watch",
            "data": "{\"directory\": \"watch\"}"
        });

        // TODO: Figure out way to start subprocess & kill it. If not possible, use python tests instead.
    }

    #[test]
    fn json_cfg_initialization() {
        let cfg = include_str!("./test_data/config.json");
        run_test(cfg, "json");
    }
}
