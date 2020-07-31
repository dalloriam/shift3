use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use protocol::{Trigger, TriggerConfiguration};

use serde_json::json;

use tempdir::TempDir;

use crate::system::TriggerSystem;

use super::mock;

#[test]
fn basic_test() {
    let sys =
        TriggerSystem::start::<mock::Dummy, mock::Dummy>(Default::default(), Default::default());
    sys.stop().unwrap();
}

#[test]
fn in_memory_full_loop() {
    let watched_directory = TempDir::new("shift3_ut_watch").unwrap();

    let watched_dir_path = watched_directory.path().to_string_lossy().to_string();
    let trigger_config = TriggerConfiguration {
        id: 42,
        rule: 42,
        trigger_type: String::from("directory_watch"),
        data: serde_json::to_string(&json!({ "directory": watched_dir_path })).unwrap(),
    };

    let cfg_loader = mock::InMemoryConfigLoader::new(vec![trigger_config]);
    let queue_writer = Arc::new(Mutex::new(mock::InMemoryQueueWriter::new()));

    let system = TriggerSystem::start(cfg_loader, queue_writer.clone());

    thread::sleep(time::Duration::from_millis(100)); // Give the system a chance to boot.

    // Add a file in the watched directory.
    let file = watched_directory.path().join("some_file.txt");
    fs::write(file, "bing bong").unwrap();

    thread::sleep(time::Duration::from_millis(200)); // Give the system a chance to pickup on the change.

    system.stop().unwrap();

    // Now that the system is stopped, make sure our new file was picked up & put in queue.
    let mut queue_guard = queue_writer.lock().unwrap();
    let queue_ref = &mut (*queue_guard);
    assert_eq!(queue_ref.queue.len(), 1);
    assert_eq!(
        queue_ref.queue.first().unwrap(),
        &Trigger {
            rule: 42,
            trigger_type: String::from("directory_watch"),
            data: serde_json::to_string(&json!({
                "file_name": "some_file.txt"
            }))
            .unwrap()
        }
    );
}
