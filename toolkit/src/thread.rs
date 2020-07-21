//! Threading utilities.

use std::sync::mpsc;
use std::thread;

use snafu::{ensure, Snafu};

/// Errors returned by thread utilities.
#[derive(Debug, Snafu)]
#[allow(missing_docs)] // Otherwise, cargo will ask to document each field of each error, which is a bit overkill.
pub enum Error {
    AlreadyStopped,
    ShutdownRequestError,
    JoinError,
}

type Result<T> = std::result::Result<T, Error>;

/// An interruptible thread.
///
/// Intended for long-running processes that can be stopped externally.
pub struct StoppableThread<T> {
    join_handle: Option<thread::JoinHandle<T>>,
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
            join_handle: Some(join_handle),
            tx_stop,
        }
    }

    /// Joins the thread.
    ///
    /// This method sends the stop signal to the thread and waits for it to complete.
    pub fn join(&mut self) -> Result<T> {
        ensure!(self.join_handle.is_some(), AlreadyStopped);

        let handle = self.join_handle.take().unwrap(); // safe because of ensure.

        ensure!(self.tx_stop.send(()).is_ok(), ShutdownRequestError);

        let join_result = handle.join();
        ensure!(join_result.is_ok(), JoinError);

        Ok(join_result.unwrap())
    }
}

impl<T> Drop for StoppableThread<T> {
    fn drop(&mut self) {
        if self.join_handle.is_some() {
            let handle = self.join_handle.take().unwrap(); // safe because of if
            if self.tx_stop.send(()).is_ok() {
                if !handle.join().is_ok() {
                    log::error!("Join error");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time;

    use super::StoppableThread;

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

    #[test]
    fn test_spawn_no_return() {
        const SLEEP_MS: u64 = 200;
        let mut handle = StoppableThread::spawn(|rx| do_something_forever(rx, SLEEP_MS));

        let time_before = time::Instant::now();
        let r = handle.join();
        let time_after = time::Instant::now();

        assert!(r.is_ok());
        assert!(time_after.duration_since(time_before).as_millis() < 2 * SLEEP_MS as u128);
    }

    #[test]
    fn test_spawn_result() {
        let mut handle = StoppableThread::spawn(do_something_and_count);
        thread::sleep(time::Duration::from_millis(150));
        let r = handle.join().unwrap();
        assert!(r >= 15);
    }
}
