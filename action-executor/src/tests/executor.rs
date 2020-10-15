use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use protocol::ActionManifest;

use crate::system::ExecutorSystem;

use super::mock;

#[test]
fn basic_test() {
    let sys = ExecutorSystem::start(Box::from(mock::Dummy::default()));
    sys.terminate().unwrap();
}

#[test]
fn in_memory_full_loop() {
    let queue_reader = Box::from(Arc::new(Mutex::new(mock::InMemoryReader::default())));

    // Push a bit of stuff in our queue before starting the system.
    for i in 0..10 {
        let mut guard = queue_reader.lock().unwrap();
        (*guard).incoming_queue.push(ActionManifest {
            data: i.to_string(),
            action_type: String::from("bing"),
            rule: 1,
        });
    }

    let sys = ExecutorSystem::start(queue_reader.clone());
    thread::sleep(time::Duration::from_millis(100)); // Give the system a chance to boot.

    sys.terminate().unwrap();

    let guard = queue_reader.lock().unwrap();
    let reader_ref = &(*guard);

    // We just make sure all messages were pulled & acknowledged. Since we don't use an
    // action integration that exists, we won't validate behavior too much.
    assert!(reader_ref.incoming_queue.is_empty());
    assert_eq!(reader_ref.received_acks.len(), 10)
}