use attohttpc::body::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fmt;
use std::sync::{Arc, Weak};

mod collector;

#[derive(Debug)]
pub struct Daemon {
    inner: Arc<ApplicationInner>,
}

impl Daemon {
    pub fn new(name: &str, license: &str) -> Self {
        crate::collector::connect_attempt(license);

        // TODO: validation
        Daemon {
            inner: Arc::new(ApplicationInner::new(name, license)),
        }
    }

    pub fn application(&self) -> Application {
        Application {
            inner: Arc::downgrade(&self.inner),
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
