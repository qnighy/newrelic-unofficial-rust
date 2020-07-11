use std::fmt;
use std::sync::{Arc, Weak};
use std::thread::{self, JoinHandle};

mod collector;
mod connect_reply;

#[derive(Debug)]
pub struct Daemon {
    inner: Arc<ApplicationInner>,
    handle: Option<JoinHandle<()>>,
}

impl Daemon {
    pub fn new(name: &str, license: &str) -> Self {
        // TODO: validation
        let inner = Arc::new(ApplicationInner::new(name, license));
        let handle = {
            let inner = inner.clone();
            thread::spawn(move || {
                inner.run();
            })
        };

        Daemon {
            inner,
            handle: Some(handle),
        }
    }

    pub fn application(&self) -> Application {
        Application {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl std::ops::Drop for Daemon {
    fn drop(&mut self) {
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

struct ApplicationInner {
    name: String,
    license: String,
}

impl ApplicationInner {
    fn new(name: &str, license: &str) -> Self {
        ApplicationInner {
            name: name.to_owned(),
            license: license.to_owned(),
        }
    }

    fn run(self: &Arc<Self>) {
        crate::collector::connect_attempt(&self.name, &self.license).unwrap();
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
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let license = std::env::var("NEW_RELIC_LICENSE_KEY").unwrap();
        Daemon::new("rust-test", &license);
    }
}
