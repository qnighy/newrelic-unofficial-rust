// Copyright 2020 New Relic Corporation. (for the original go-agent)
// Copyright 2020 Masaki Hara.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct AgentRunId(pub(crate) String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PreconnectReply {
    pub(crate) redirect_host: String,
    // pub(crate) security_policies: SecurityPolicies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConnectReply {
    pub(crate) agent_run_id: AgentRunId,
    pub(crate) request_headers_map: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_payload_size_in_bytes: Option<usize>,
    pub(crate) entity_guid: String,

    // Transaction Name Modifiers
    // transaction_segment_terms: SegmentRules,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) transaction_name_rules: Vec<MetricRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) url_rules: Vec<MetricRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) metric_name_rules: Vec<MetricRule>,

    // Cross Process
    pub(crate) encoding_key: String,
    pub(crate) cross_process_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) trusted_account_set: Vec<i32>,

    // Settings
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) web_transactions_apdex: HashMap<String, f64>,
    pub(crate) apdex_t: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) collect_analytics_events: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) collect_custom_events: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) collect_traces: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) collect_errors: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) collect_error_events: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) collect_span_events: Option<bool>,

    // RUM
    pub(crate) js_agent_loader: String,
    pub(crate) beacon: String,
    pub(crate) browser_key: String,
    pub(crate) application_id: String,
    pub(crate) error_beacon: String,
    pub(crate) js_agent_file: String,

    pub(crate) messages: Vec<Message>,

    // BetterCAT/Distributed Tracing
    pub(crate) account_id: String,
    pub(crate) trusted_account_key: String,
    pub(crate) primary_application_id: String,
    pub(crate) sampling_target: u64,
    pub(crate) sampling_target_period_in_seconds: i32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent_config: Option<ServerSideConfig>,

    pub(crate) event_harvest_config: EventHarvestConfig,

    #[serde(flatten)]
    pub(crate) remain: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MetricRule {
    pub(crate) ignore: bool,
    pub(crate) each_segment: bool,
    pub(crate) replace_all: bool,
    pub(crate) terminate_chain: bool,
    pub(crate) eval_order: i32,
    pub(crate) replacement: String,
    pub(crate) match_expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Message {
    pub(crate) message: String,
    pub(crate) level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ServerSideConfig {
    #[serde(rename = "transaction_tracer.enabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_tracer_enabled: Option<bool>,
    #[serde(rename = "transaction_tracer.transaction_threshold")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_tracer_threshold: Option<TransactionTracerThreshold>,
    #[serde(rename = "error_collector.enabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) error_collector_enabled: Option<bool>,
    #[serde(rename = "error_collector.ignore_status_codes")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) error_collector_ignore_status_codes: Vec<i32>,
    #[serde(rename = "cross_application_tracer.enabled")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cross_application_tracer_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum TransactionTracerThreshold {
    Value(f64),
    // TODO: the string must always be "apdex_f"
    ApdexF(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventHarvestConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) report_period_ms: Option<u32>,
    pub(crate) harvest_limits: HarvestLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarvestLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) analytic_event_data: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) custom_event_data: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) error_event_data: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) span_event_data: Option<u32>,
}
