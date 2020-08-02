// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use parking_lot::Mutex;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Weak};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::app_run::AppRun;
pub use crate::config::Config;
use crate::harvest::Harvest;
pub use crate::transaction::Transaction;

mod analytics_events;
mod app_run;
mod collector;
pub mod config;
mod connect_reply;
mod harvest;
mod limits;
mod metrics;
mod transaction;
mod utilization;

#[derive(Debug)]
pub struct Daemon {
    inner: Arc<ApplicationInner>,
    handle: Option<JoinHandle<()>>,
}

impl Daemon {
    pub fn new(app_name: &str, license: &str) -> Result<Self, crate::config::ConfigError> {
        Self::from_config(&Config::new(app_name, license))
    }

    pub(crate) fn from_config(config: &Config) -> Result<Self, crate::config::ConfigError> {
        config.validate()?;

        let (wake, wake_recv) = mpsc::sync_channel::<()>(1);
        let inner = Arc::new(ApplicationInner::new(&config.app_name, &config.license, wake));
        let handle = {
            let inner = inner.clone();
            thread::spawn(move || {
                inner.run(wake_recv);
            })
        };

        Ok(Daemon {
            inner,
            handle: Some(handle),
        })
    }

    pub fn application(&self) -> Application {
        Application {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl std::ops::Drop for Daemon {
    fn drop(&mut self) {
        self.inner.shutdown.store(true, Relaxed);
        let _ = self.inner.wake.try_send(());
        if let Some(handle) = self.handle.take() {
            let result = handle.join();
            if let Err(e) = result {
                // TODO: logging
                eprintln!("NewRelic daemon failed: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    inner: Weak<ApplicationInner>,
}

impl Application {
    pub fn start_transaction(&self, name: &str) -> Transaction {
        Transaction::new(&self.inner, name)
    }
}

struct ApplicationInner {
    name: String,
    license: String,
    state: Mutex<AppState>,
    shutdown: AtomicBool,
    wake: SyncSender<()>,
}

#[derive(Debug)]
enum AppState {
    Init,
    Running { run: AppRun, harvest: Harvest },
    Dead,
}

impl ApplicationInner {
    fn new(name: &str, license: &str, wake: SyncSender<()>) -> Self {
        ApplicationInner {
            name: name.to_owned(),
            license: license.to_owned(),
            state: Mutex::new(AppState::Init),
            shutdown: AtomicBool::new(false),
            wake,
        }
    }

    fn run(self: &Arc<Self>, wake_recv: Receiver<()>) {
        // TODO: handle errors
        let run = crate::collector::connect_attempt(&self.name, &self.license).unwrap();
        eprintln!("run = {:#?}", run);
        let harvest = Harvest::new(&run);
        {
            let mut state = self.state.lock();
            *state = AppState::Running {
                run,
                harvest: harvest,
            };
        }
        while !self.shutdown.load(Relaxed) {
            let result = wake_recv.recv_timeout(Duration::from_secs(1));
            match result {
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {
                    let mut state = self.state.lock();
                    if let AppState::Running { run, harvest } = &mut *state {
                        harvest.harvest(run, false);
                    }
                }
                Ok(()) => {}
            }
        }
        eprintln!("shutting down...");
        let mut old_state = {
            let mut state = self.state.lock();
            std::mem::replace(&mut *state, AppState::Dead)
        };
        if let AppState::Running { run, harvest } = &mut old_state {
            harvest.harvest(run, true);
        }
    }
}

impl fmt::Debug for ApplicationInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct LicensePlaceholder;
        impl fmt::Debug for LicensePlaceholder {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("<filtered>")
            }
        }
        f.debug_struct("Application")
            .field("name", &self.name)
            .field("license", &LicensePlaceholder)
            .field("state", &self.state)
            .field("shutdown", &self.shutdown)
            .field("wake", &self.wake)
            .finish()
    }
}
