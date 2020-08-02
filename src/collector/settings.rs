use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::config::{UtilizationConfig, Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Settings {
    app_name: String,
    // license: String,
    enabled: bool,
    labels: HashMap<String, String>,
    host_display_name: Option<String>,
    utilization: UtilizationSettings,
    host: Option<String>,
}

impl Settings {
    pub(super) fn new(config: &Config) -> Self {
        Self {
            app_name: config.app_name.clone(),
            enabled: config.enabled,
            labels: config.labels.clone(),
            host_display_name: config.host_display_name.clone(),
            utilization: UtilizationSettings::new(&config.utilization),
            host: config.host.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UtilizationSettings {
    pub detect_docker: bool,
    pub detect_kubernetes: bool,
}

impl UtilizationSettings {
    fn new(config: &UtilizationConfig) -> Self {
        Self {
            detect_docker: config.detect_docker,
            detect_kubernetes: config.detect_kubernetes,
        }
    }
}