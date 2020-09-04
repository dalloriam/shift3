/// Defines a struct that can be stopped once.
pub trait Stop {
    /// The error to be returned by a call to stop.
    type Error;

    /// Stops the instance, consuming it in the process.
    fn stop(self: Box<Self>) -> Result<(), Self::Error>;
}
