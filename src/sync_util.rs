use parking_lot::{Condvar, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("shutdown")]
pub(crate) struct ShutdownError;

#[derive(Debug)]
pub(crate) struct Shutdown {
    shutdown: Mutex<bool>,
    cond: Condvar,
}

impl Shutdown {
    pub(crate) fn new() -> Self {
        Self {
                shutdown: Mutex::new(false),
                cond: Condvar::new(),
            }
    }

    pub(crate) fn shutdown(&self) {
        let mut shutdown = self.shutdown.lock();
        *shutdown = true;
        self.cond.notify_all();
    }

    pub(crate) fn sleep(&self, duration: Duration) -> Result<(), ShutdownError> {
        let timeout = Instant::now() + duration;
        let mut shutdown = self.shutdown.lock();
        while !*shutdown {
            let result = self.cond.wait_until(&mut shutdown, timeout);
            if result.timed_out() {
                return Ok(());
            }
        }
        Err(ShutdownError)
    }
}
