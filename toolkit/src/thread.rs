use std::sync::mpsc;
use std::thread;

use snafu::{ensure, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    AlreadyStopped,
    ShutdownRequestError,
    JoinError,
}

type Result<T> = std::result::Result<T, Error>;

pub struct StoppableThread<T> {
    pub join_handle: Option<thread::JoinHandle<T>>,
    pub tx_stop: mpsc::Sender<()>,
}

impl<T> StoppableThread<T>
where
    T: Send + 'static,
{
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
