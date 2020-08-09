use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::{Config, TransactionTracerConfig, UtilizationConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Settings {
    app_name: String,
    // license: String,
    enabled: bool,
    labels: HashMap<String, String>,
    host_display_name: Option<String>,
    transaction_tracer: TransactionTracerSettings,
    utilization: UtilizationSettings,
    host: Option<String>,
    // Tell who we are
    unofficial_agent_repository: String,
}

impl Settings {
    pub(super) fn new(config: &Config) -> Self {
        Self {
            app_name: config.app_name.clone(),
            enabled: config.enabled,
            labels: config.labels.clone(),
            host_display_name: config.host_display_name.clone(),
            transaction_tracer: TransactionTracerSettings::new(&config.transaction_tracer),
            utilization: UtilizationSettings::new(&config.utilization),
            host: config.host.clone(),
            unofficial_agent_repository: "https://github.com/qnighy/newrelic-unofficial-rust"
                .to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TransactionTracerSettings {
    enabled: bool,
}

impl TransactionTracerSettings {
    fn new(config: &TransactionTracerConfig) -> Self {
        Self {
            enabled: config.enabled,
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
