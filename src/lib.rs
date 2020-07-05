use attohttpc::body::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Daemon {
    inner: Arc<ApplicationInner>,
}

impl Daemon {
    pub fn new(name: &str, license: &str) -> Self {
        let compressed = {
            let mut stream = GzEncoder::new(Vec::<u8>::new(), Compression::default());
            serde_json::to_writer(
                &mut stream,
                &vec![PreconnectRequest {
                    security_policies_token: "".to_owned(),
                    high_security: false,
                }],
            )
            .unwrap();
            stream.finish().unwrap()
        };
        eprintln!("compressed = {:?}", compressed);
        let resp =
            attohttpc::post("https://collector.newrelic.com/agent_listener/invoke_raw_method")
                .param("marshal_format", "json")
                .param("protocol_version", "17")
                .param("method", "preconnect")
                .param("license_key", license)
                .header("Accept-Encoding", "identity, deflate")
                .header("Content-Type", "application/octet-stream")
                .header("User-Agent", "NewRelic-Rust-Agent-Unofficial/0.1.0")
                .header("Content-Encoding", "gzip")
                .body(Bytes(compressed))
                .send();

        let resp = resp.unwrap();
        eprintln!("resp = {:#?}", resp);
        let body = resp.text().unwrap();
        eprintln!("body = {:?}", body);

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreconnectRequest {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub security_policies_token: String,
    pub high_security: bool,
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
