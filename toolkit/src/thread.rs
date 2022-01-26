//! Threading utilities.

use std::sync::mpsc;
use std::thread;

use snafu::{ensure, Snafu};

/// Errors returned by thread utilities.
#[derive(Debug, PartialEq, Snafu)]
#[allow(missing_docs)] // Otherwise, cargo will ask to document each field of each error, which is a bit overkill.
pub enum Error {
    AlreadyStopped,
    ShutdownRequest,
    Join,
}

type Result<T> = std::result::Result<T, Error>;

/// JoinHolder holds the join handle of a thread for which
/// stop was requested.
pub struct JoinHolder<T> {
    handle: thread::JoinHandle<T>,
}

impl<T> JoinHolder<T>
where
    T: Send + 'static,
{
    /// Returns a new join holder from a thread join handle.
    pub fn new(handle: thread::JoinHandle<T>) -> Self {
        Self { handle }
    }

    /// Joins the holder and returns the return value of the thread.
    pub fn join(self) -> Result<T> {
        let join_result = self.handle.join();
        ensure!(join_result.is_ok(), JoinSnafu);

        Ok(join_result.unwrap())
    }
}

/// An interruptible thread.
///
/// Intended for long-running processes that can be stopped externally.
pub struct StoppableThread<T> {
    join_handle: thread::JoinHandle<T>,
    tx_stop: mpsc::Sender<()>,
}

impl<T> StoppableThread<T>
where
    T: Send + 'static,
{
    /// Spawns a stoppable thread with the provided function.
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce(mpsc::Receiver<()>) -> T,
        F: Send + 'static,
    {
        let (tx_stop, rx_stop) = mpsc::channel();
        let join_handle = thread::spawn(move || f(rx_stop));

        Self {
            join_handle,
            tx_stop,
        }
    }

    /// Sends a stop signal to the thread, returns a join holding object.
    pub fn stop(self) -> Result<JoinHolder<T>> {
        let handle = self.join_handle;

        ensure!(self.tx_stop.send(()).is_ok(), ShutdownRequestSnafu);

        Ok(JoinHolder::new(handle))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time;

    use super::{Error, StoppableThread};

    fn do_something_forever(stop_rx: mpsc::Receiver<()>, sleep_ms: u64) {
        loop {
            thread::sleep(time::Duration::from_millis(sleep_ms));
            if stop_rx.try_recv().is_ok() {
                break;
            }
        }
    }

    fn do_something_and_count(stop_rx: mpsc::Receiver<()>) -> u64 {
        let mut counter = 0;
        loop {
            thread::sleep(time::Duration::from_millis(10));
            counter += 1;

            if stop_rx.try_recv().is_ok() {
                break;
            }
        }

        counter
    }

    fn do_something_and_stop(_: mpsc::Receiver<()>) {}

    #[test]
    fn test_spawn_no_return() {
        const SLEEP_MS: u64 = 200;
        let handle = StoppableThread::spawn(|rx| do_something_forever(rx, SLEEP_MS));

        let time_before = time::Instant::now();
        let r = handle.stop().unwrap().join();
        let time_after = time::Instant::now();

        assert!(r.is_ok());
        assert!(time_after.duration_since(time_before).as_millis() < 2 * SLEEP_MS as u128);
    }

    #[test]
    fn test_spawn_result() {
        let handle = StoppableThread::spawn(do_something_and_count);
        thread::sleep(time::Duration::from_millis(200));
        let r = handle.stop().unwrap().join().unwrap();
        assert!(r >= 15);
    }

    #[test]
    fn test_thread_crash() {
        let handle = StoppableThread::spawn(do_something_and_stop);
        thread::sleep(time::Duration::from_millis(100));
        if let Err(e) = handle.stop() {
            assert_eq!(e, Error::ShutdownRequest);
        } else {
            panic!("error expected")
        }
    }
}
