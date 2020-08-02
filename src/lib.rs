// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::app_run::AppRun;
use crate::collector::RpmError;
pub use crate::config::Config;
use crate::harvest::Harvest;
use crate::sync_util::Shutdown;
pub use crate::transaction::Transaction;

mod analytics_events;
mod app_run;
mod collector;
pub mod config;
mod connect_reply;
mod harvest;
mod limits;
mod metrics;
mod sync_util;
mod transaction;
mod utilization;

#[derive(Debug)]
pub struct Daemon {
    app: Application,
    handle: Option<JoinHandle<()>>,
}

impl Daemon {
    pub fn new(app_name: &str, license: &str) -> Result<Self, crate::config::ConfigError> {
        Self::from_config(&Config::new(app_name, license))
    }

    pub(crate) fn from_config(config: &Config) -> Result<Self, crate::config::ConfigError> {
        config.validate()?;

        let app = Application::new(&config);
        if !config.enabled {
            return Ok(Daemon { app, handle: None });
        }
        let handle = {
            let inner = app.inner.clone();
            thread::spawn(move || {
                inner.run();
            })
        };

        Ok(Daemon {
            app,
            handle: Some(handle),
        })
    }

    pub fn application(&self) -> &Application {
        &self.app
    }

    pub fn start_transaction(&self, name: &str) -> Transaction {
        self.app.start_transaction(name)
    }

    pub fn shutdown(&self) {
        self.app.shutdown()
    }
}

impl std::ops::Drop for Daemon {
    fn drop(&mut self) {
        self.shutdown();
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
    inner: Arc<ApplicationInner>,
}

impl Application {
    fn new(config: &Config) -> Self {
        Self {
            inner: Arc::new(ApplicationInner::new(config)),
        }
    }

    pub fn start_transaction(&self, name: &str) -> Transaction {
        Transaction::new(&self.inner, name)
    }

    pub fn shutdown(&self) {
        self.inner.shutdown.shutdown();
    }
}

#[derive(Debug)]
struct ApplicationInner {
    config: Config,
    state: Mutex<AppState>,
    shutdown: Shutdown,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum AppState {
    Init,
    Running { run: Arc<AppRun>, harvest: Harvest },
    Dead,
}

impl ApplicationInner {
    fn new(config: &Config) -> Self {
        let state = if config.enabled {
            AppState::Init
        } else {
            AppState::Dead
        };
        ApplicationInner {
            config: config.clone(),
            state: Mutex::new(state),
            shutdown: Shutdown::new(),
        }
    }

    fn run(self: &Arc<Self>) {
        let mut attempt: u32 = 0;
        loop {
            let e = match crate::collector::connect_attempt(&self.config) {
                Ok(run) => {
                    attempt = 0;
                    match self.run1(run) {
                        Ok(void) => match void {},
                        Err(e @ RpmError::Shutdown(..)) => {
                            self.shutdown();
                            e
                        }
                        Err(e) => e,
                    }
                }
                Err(e) => {
                    attempt = attempt.saturating_add(1);
                    e
                }
            };
            if e.is_disconnect() {
                log::error!("application disconnected: {}", e);
                break;
            } else {
                let backoff_time = connect_backoff_time(attempt);
                if let Err(_shutdown) = self.shutdown.sleep(backoff_time) {
                    break;
                }
            }
        }
        {
            let mut state = self.state.lock();
            *state = AppState::Dead;
        }
    }

    fn run1(self: &Arc<Self>, run: AppRun) -> Result<Void, RpmError> {
        eprintln!("run = {:#?}", run);
        let harvest = Harvest::new(&run);
        {
            let mut state = self.state.lock();
            *state = AppState::Running {
                run: Arc::new(run),
                harvest,
            };
        }
        loop {
            self.shutdown.sleep(Duration::from_secs(1))?;
            // Only invoke Harvest::ready() during locking.
            let ready = {
                let mut state = self.state.lock();
                if let AppState::Running { run, harvest } = &mut *state {
                    Some((Arc::clone(run), harvest.ready(run, false)))
                } else {
                    None
                }
            };
            // Do harvest after unlock
            if let Some((run, ready)) = ready {
                let result = ready.harvest(&run);
                if let Err(e) = result {
                    if e.is_disconnect() || e.is_restart_exception() {
                        return Err(e);
                    } else {
                        log::warn!("harvest failure: {}", e);
                    }
                }
            }
        }
    }

    fn shutdown(self: &Arc<Self>) {
        eprintln!("shutting down...");
        let mut old_state = {
            let mut state = self.state.lock();
            std::mem::replace(&mut *state, AppState::Dead)
        };
        if let AppState::Running { run, harvest } = &mut old_state {
            let ready = harvest.ready(run, true);
            let result = ready.harvest(run);
            if let Err(e) = result {
                log::warn!("harvest failure: {}", e);
            }
        }
    }
}

enum Void {}

fn connect_backoff_time(attempt: u32) -> Duration {
    const CONNECT_BACKOFF_TIMES: &[Duration] = &[
        Duration::from_secs(15),
        Duration::from_secs(15),
        Duration::from_secs(30),
        Duration::from_secs(60),
        Duration::from_secs(120),
        Duration::from_secs(300),
    ];
    const BACKOFF_REPEAT: Duration = CONNECT_BACKOFF_TIMES[CONNECT_BACKOFF_TIMES.len() - 1];
    CONNECT_BACKOFF_TIMES
        .get(attempt as usize)
        .copied()
        .unwrap_or(BACKOFF_REPEAT)
}
