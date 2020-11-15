use std::collections::HashMap;
use std::time;
use std::{sync::Arc, sync::Mutex, thread};

use protocol::{ActionManifest, Rule, Trigger};

use crate::interpreter::TriggerInterpreter;

use super::mock;

#[test]
fn basic_test() {
    let sys = TriggerInterpreter::start(
        Box::new(mock::Dummy::default()),
        Box::new(mock::Dummy::default()),
        Box::new(mock::Dummy::default()),
    );

    sys.terminate().unwrap();
}

#[test]
fn in_memory_full_loop() {
    let rule = Rule {
        trigger_config_id: 1,
        action_config: String::from(
            "{\"body\": \"New file: {{file_name}}\", \"title\": \"ShifTTT: New File Created\"}",
        ),
        action_type: String::from("notify"),
    };
    let mut action_configs = HashMap::new();
    action_configs.insert(1, rule.clone());

    let file_name = "test";
    let triggers = vec![Trigger {
        rule: 1,
        trigger_type: String::from("file"),
        data: String::from(format!("{{\"file_name\": \"{}\"}}", file_name)),
    }];

    let cfg_loader = Box::new(mock::InMemoryActionConfigReader::new(action_configs));
    let queue_reader = Box::new(Arc::new(Mutex::new(mock::InMemoryQueueReader::new(
        triggers,
    ))));
    let queue_writer = Box::new(Arc::new(Mutex::new(mock::InMemoryQueueWriter::new())));

    let system = TriggerInterpreter::start(queue_reader.clone(), cfg_loader, queue_writer.clone());

    thread::sleep(time::Duration::from_millis(100)); // Give the system a chance to boot.

    system.terminate().unwrap();

    // Now that the system is stopped, make sure the trigger was properly interpreter and put into the action executer queue.
    let mut queue_writer_guard = queue_writer.lock().unwrap();
    let queue_writer_ref = &mut (*queue_writer_guard);

    let mut queue_reader_guard = queue_reader.lock().unwrap();
    let queue_reader_ref = &mut (*queue_reader_guard);

    // Makes sure the trigger queue is empty
    assert_eq!(queue_reader_ref.queue.len(), 0);

    // Makes sure the trigger was properly interpreted
    assert_eq!(queue_writer_ref.queue.len(), 1);
    assert_eq!(
        queue_writer_ref.queue.first().unwrap(),
        &ActionManifest {
            rule: 1,
            action_type: rule.action_type.clone(),
            data: String::from(format!(
                "{{\"body\": \"New file: {}\", \"title\": \"ShifTTT: New File Created\"}}",
                file_name
            ))
        }
    );
}

#[test]
fn in_memory_action_config_missing() {
    let action_configs = HashMap::new();

    let file_name = "test";
    let triggers = vec![Trigger {
        rule: 1,
        trigger_type: String::from("file"),
        data: String::from(format!("{{\"file_name\": \"{}\"}}", file_name)),
    }];

    let cfg_loader = Box::new(mock::InMemoryActionConfigReader::new(action_configs));
    let queue_reader = Box::new(Arc::new(Mutex::new(mock::InMemoryQueueReader::new(
        triggers,
    ))));
    let queue_writer = Box::new(Arc::new(Mutex::new(mock::InMemoryQueueWriter::new())));

    let system = TriggerInterpreter::start(queue_reader.clone(), cfg_loader, queue_writer.clone());

    thread::sleep(time::Duration::from_millis(100)); // Give the system a chance to boot.

    system.terminate().unwrap();

    let mut queue_writer_guard = queue_writer.lock().unwrap();
    let queue_writer_ref = &mut (*queue_writer_guard);

    let mut queue_reader_guard = queue_reader.lock().unwrap();
    let queue_reader_ref = &mut (*queue_reader_guard);

    // Makes sure the trigger wasn't acknowledged.
    assert_eq!(queue_reader_ref.ack_count(), 0);

    // Makes sure the trigger was not interpreted
    assert_eq!(queue_writer_ref.queue.len(), 0);
}
