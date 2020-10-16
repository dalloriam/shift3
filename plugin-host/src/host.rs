use std::path::{Path, PathBuf};

use libloading::Library;

use snafu::{ensure, ResultExt, Snafu};

use plugin_core::{ActionPlugin, Error as PlugError, Plugin, TriggerPlugin, PLUGIN_INIT_SYMBOL};

type PluginLoadFn = fn() -> Box<Plugin>;

#[derive(Debug, Snafu)]
pub enum Error {
    FailedToLoadLibrary { source: libloading::Error },
    InternalPluginError { source: PlugError },
    MissingSymbol { source: libloading::Error },
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

#[derive(Default)]
pub struct PluginHost {
    loaded_plugins: Vec<PluginHandle>,
}

impl PluginHost {
    pub fn new() -> PluginHost {
        Self::default()
    }

    pub fn add_plugin<P: AsRef<Path>>(&mut self, library_path: P) -> Result<()> {
        let plug_handle = PluginHandle::load(library_path)?;
        self.loaded_plugins.push(plug_handle);
        Ok(())
    }
}
