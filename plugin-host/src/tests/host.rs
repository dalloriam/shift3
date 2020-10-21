use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tempfile::tempdir;

use crate::PluginHost;

#[test]
fn plugin_loading() {
    const PLUGIN_SO_DATA: &[u8] = include_bytes!("test_data/libdirectory_watch.so");

    // Write the plugin to a temp file.
    let temp_dir = tempdir().unwrap();
    let plugin_path = temp_dir.path().join("plugin.so");
    {
        let mut plugin_file = fs::File::create(&plugin_path).unwrap();
        plugin_file.write_all(PLUGIN_SO_DATA).unwrap();
    }

    // Create a plugin host.
    let search_paths = vec![PathBuf::from(temp_dir.path())];
    let host = PluginHost::initialize(&search_paths).unwrap();
    assert_eq!(host.get_action_plugins().len(), 0);
    assert_eq!(host.get_trigger_plugins().len(), 1);
}
