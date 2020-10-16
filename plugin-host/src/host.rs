use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use libloading::Library;

use snafu::{ensure, ResultExt, Snafu};

use plugin_core::{ActionPlugin, Error as PlugError, Plugin, TriggerPlugin, PLUGIN_INIT_SYMBOL};

#[cfg(unix)]
const PLUGIN_EXTENSION: &str = "so";

#[cfg(windows)]
const PLUGIN_EXTENSION: &str = "dll";

type PluginLoadFn = fn() -> Box<Plugin>;

#[derive(Debug, Snafu)]
pub enum Error {
    FailedToLoadLibrary { source: libloading::Error },
    FailedToOpenSearchPath { source: io::Error },
    InternalPluginError { source: PlugError },
    MissingSymbol { source: libloading::Error },
    SearchPathDoesNotExist,
    SearchPathIsAFile,
}

type Result<T> = std::result::Result<T, Error>;

struct PluginHandle {
    plugin: Plugin,
    library: Library,
    path: PathBuf,
}

impl PluginHandle {
    pub fn load<P: AsRef<Path>>(library_path: P) -> Result<PluginHandle> {
        let library = Library::new(library_path.as_ref()).context(FailedToLoadLibrary)?;

        let plugin_loader: libloading::Symbol<PluginLoadFn> = unsafe {
            library
                .get(PLUGIN_INIT_SYMBOL.as_bytes())
                .context(MissingSymbol)?
        };

        let plugin_box = plugin_loader();

        Ok(PluginHandle {
            plugin: *plugin_box,
            library,
            path: PathBuf::from(library_path.as_ref()),
        })
    }
}

pub struct PluginHost {
    loaded_plugins: Vec<PluginHandle>,
    search_paths: Vec<PathBuf>,
}

impl PluginHost {
    pub fn initialize(search_paths: &[PathBuf]) -> Result<PluginHost> {
        let mut host = PluginHost {
            loaded_plugins: Vec::new(),
            search_paths: Vec::from(search_paths),
        };

        host.search()?;

        Ok(host)
    }

    fn add_plugin<P: AsRef<Path>>(&mut self, library_path: P) -> Result<()> {
        let plug_handle = PluginHandle::load(library_path.as_ref())?;
        self.loaded_plugins.push(plug_handle);
        log::info!("loaded plugin: {}", library_path.as_ref().display());
        Ok(())
    }

    fn search(&mut self) -> Result<()> {
        log::info!("beginning plugin refresh");
        let paths_copy = self.search_paths.clone();

        for path in paths_copy.into_iter() {
            ensure!(path.exists(), SearchPathDoesNotExist);
            ensure!(path.is_dir(), SearchPathIsAFile);

            for entry in fs::read_dir(path)
                .context(FailedToOpenSearchPath)?
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();

                if let Some(ext) = entry_path.extension() {
                    if ext.to_string_lossy().as_ref() == PLUGIN_EXTENSION {
                        // Load plugin from library.
                        self.add_plugin(&entry_path)?;
                    }
                }
            }
        }
        log::info!("plugin refresh complete");
        Ok(())
    }

    pub fn get_action_plugins(&self) -> Vec<Arc<Box<dyn ActionPlugin>>> {
        let mut v: Vec<Arc<Box<dyn ActionPlugin>>> = Vec::new();

        for plug_handle in self.loaded_plugins.iter() {
            for action_plug in plug_handle.plugin.actions.iter() {
                v.push(action_plug.clone());
            }
        }

        v
    }

    pub fn get_trigger_plugins(&self) -> Vec<Arc<Box<dyn TriggerPlugin>>> {
        let mut v: Vec<Arc<Box<dyn TriggerPlugin>>> = Vec::new();

        for plug_handle in self.loaded_plugins.iter() {
            for trigger_plug in plug_handle.plugin.triggers.iter() {
                v.push(trigger_plug.clone());
            }
        }

        v
    }
}
