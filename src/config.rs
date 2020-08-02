use std::collections::HashMap;
use thiserror::Error;

const LICENSE_LENGTH: usize = 40;
const APP_NAME_LIMIT: usize = 3;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("license length is not {}", LICENSE_LENGTH)]
    LicenseLength,
    #[error("app_name is required")]
    AppNameMissing,
    #[error("max of {} rollup application names", APP_NAME_LIMIT)]
    AppNameLimit,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub license: String,
    pub enabled: bool,
    pub labels: HashMap<String, String>,
    pub host_display_name: Option<String>,
    pub utilization: UtilizationConfig,
    pub host: Option<String>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: String::default(),
            license: String::default(),
            enabled: true,
            labels: HashMap::default(),
            host_display_name: None,
            utilization: UtilizationConfig::default(),
            host: None,
            __non_exhaustive: (),
        }
    }
}

impl Config {
    pub fn new(app_name: &str, license: &str) -> Self {
        Self {
            app_name: app_name.to_owned(),
            license: license.to_owned(),
            ..Self::default()
        }
    }

    pub fn start(&self) -> Result<crate::Daemon, ConfigError> {
        crate::Daemon::from_config(self)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled {
            if self.license.len() != LICENSE_LENGTH {
                return Err(ConfigError::LicenseLength);
            }
            if self.app_name.is_empty() {
                return Err(ConfigError::AppNameMissing);
            } else if self.app_name.split(",").count() > APP_NAME_LIMIT {
                return Err(ConfigError::AppNameLimit);
            }
        } else {
            if self.license.len() != LICENSE_LENGTH && !self.license.is_empty() {
                return Err(ConfigError::LicenseLength);
            }
        }
        if !self.app_name.is_empty() || self.enabled {
            if self.app_name.len() != LICENSE_LENGTH {}
        }

        Ok(())
    }

    pub fn with_app_name(self, app_name: &str) -> Self {
        Self {
            app_name: app_name.to_owned(),
            ..self
        }
    }
    pub fn with_license(self, license: &str) -> Self {
        Self {
            license: license.to_owned(),
            ..self
        }
    }

    pub fn with_enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn with_host_display_name(self, host_display_name: &str) -> Self {
        Self {
            host_display_name: Some(host_display_name.to_owned()),
            ..self
        }
    }

    pub fn with_host(self, host: &str) -> Self {
        Self {
            host: Some(host.to_owned()),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
pub struct UtilizationConfig {
    pub detect_docker: bool,
    pub detect_kubernetes: bool,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for UtilizationConfig {
    fn default() -> Self {
        Self {
            detect_docker: true,
            detect_kubernetes: true,
            __non_exhaustive: (),
        }
    }
}
